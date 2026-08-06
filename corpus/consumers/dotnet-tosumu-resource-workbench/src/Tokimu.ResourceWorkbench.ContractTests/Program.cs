using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Tokimu.ResourceWorkbench.Bridge;

try
{
    var bridgePath = Environment.GetEnvironmentVariable("TOKIMU_RESOURCE_BRIDGE");
    if (string.IsNullOrWhiteSpace(bridgePath) || !File.Exists(bridgePath))
    {
        throw new InvalidOperationException(
            "TOKIMU_RESOURCE_BRIDGE must point to the built tokimu-resource-workbench-bridge executable.");
    }

    await using var bridge = new ResourceBridgeClient(bridgePath);
    var session = await bridge.ExecuteAsync("session.create_or_open");
    AssertEqual("in_memory", session.GetProperty("mode").GetString(), "session provider mode");

    var folder = await bridge.ExecuteAsync("folder.create", new { name = "notes" });
    var folderId = folder.GetProperty("folder").GetProperty("id").GetString();
    AssertTrue(!string.IsNullOrWhiteSpace(folderId), "folder create returns a stable id");

    var payload = Convert.ToBase64String(Encoding.UTF8.GetBytes("consumer contract"));
    await bridge.ExecuteAsync("resource.put", new
    {
        parent_folder_id = folderId,
        name = "readme.txt",
        bytes_base64 = payload,
        media_type = "text/plain",
    });
    await bridge.ExecuteAsync("resource.set_visibility", new
    {
        parent_folder_id = folderId,
        name = "readme.txt",
        visibility = "hidden",
    });
    var visible = await bridge.ExecuteAsync("resource.list", new
    {
        parent_folder_id = folderId,
        visibility = "visible",
    });
    AssertEqual(0, visible.GetProperty("resources").GetArrayLength(), "hidden resource remains excluded from visible navigation");

    var fetched = await bridge.ExecuteAsync("resource.get", new
    {
        parent_folder_id = folderId,
        name = "readme.txt",
    });
    AssertEqual(payload, fetched.GetProperty("bytes_base64").GetString(), "resource bytes survive the bridge");
    AssertEqual("hidden", fetched.GetProperty("resource").GetProperty("visibility").GetString(), "visibility changes without replacing bytes");

    var summary = await bridge.ExecuteAsync("observation.summary");
    AssertEqual(2L, summary.GetProperty("folders").GetInt64(), "summary retains folder hierarchy");
    AssertEqual(1L, summary.GetProperty("resources").GetInt64(), "summary retains resource count");

    try
    {
        await bridge.ExecuteAsync("tosumu.inspect");
        throw new InvalidOperationException("unknown Tokimu command should fail");
    }
    catch (ResourceBridgeCommandException exception)
    {
        AssertEqual("command.unknown", exception.Kind, "bridge classifies unknown command failure");
    }

    await RunTransportFailureChecksAsync();
    await RunTosumuDurabilityCheckAsync(bridgePath);
    await WriteProviderOperationEvidenceAsync(bridgePath);
    await WriteCompressionProviderEvidenceAsync(bridgePath);
    await WriteArchiveProviderEvidenceAsync(bridgePath);
    Console.WriteLine("Tokimu Resource Workbench bridge contract checks passed.");
}
catch (Exception exception)
{
    Console.Error.WriteLine($"Tokimu Resource Workbench bridge contract checks failed: {exception}");
    Environment.ExitCode = 1;
}

static void AssertTrue(bool value, string description)
{
    if (!value)
    {
        throw new InvalidOperationException($"Assertion failed: {description}");
    }
}

static void AssertEqual<T>(T? expected, T? actual, string description)
{
    if (!EqualityComparer<T?>.Default.Equals(expected, actual))
    {
        throw new InvalidOperationException($"Assertion failed: {description}. Expected {expected}, observed {actual}.");
    }
}

static async Task RunTransportFailureChecksAsync()
{
    var temporaryDirectory = Path.Combine(Path.GetTempPath(), $"tokimu-resource-bridge-{Guid.NewGuid():N}");
    Directory.CreateDirectory(temporaryDirectory);
    try
    {
        var malformed = WriteCommand(temporaryDirectory, "malformed", "echo not-json");
        await using (var malformedBridge = new ResourceBridgeClient(malformed))
        {
            await AssertThrowsAsync<ResourceBridgeProtocolException>(
                () => malformedBridge.ExecuteAsync("observation.summary"),
                "malformed bridge output remains a transport-owned failure");
        }

        var stderr = WriteCommand(temporaryDirectory, "stderr", "echo intentional stderr 1>&2\r\nexit /b 9");
        await using (var stderrBridge = new ResourceBridgeClient(stderr))
        {
            var exception = await AssertThrowsAsync<ResourceBridgeTransportException>(
                () => stderrBridge.ExecuteAsync("observation.summary"),
                "bridge stderr remains a host transport diagnostic");
            AssertTrue(exception.Message.Contains("intentional stderr", StringComparison.Ordinal), "stderr is retained in transport failure");
        }

        var delayed = WriteCommand(temporaryDirectory, "delayed", "ping 127.0.0.1 -n 6 > nul");
        await using (var delayedBridge = new ResourceBridgeClient(delayed))
        using (var cancellation = new CancellationTokenSource(TimeSpan.FromMilliseconds(25)))
        {
            await AssertThrowsAsync<OperationCanceledException>(
                () => delayedBridge.ExecuteAsync("observation.summary", cancellationToken: cancellation.Token),
                "caller cancellation stops a pending host bridge request");
        }
    }
    finally
    {
        Directory.Delete(temporaryDirectory, recursive: true);
    }
}

static async Task RunTosumuDurabilityCheckAsync(string bridgePath)
{
    var temporaryDirectory = Path.Combine(Path.GetTempPath(), $"tokimu-resource-tosumu-{Guid.NewGuid():N}");
    Directory.CreateDirectory(temporaryDirectory);
    var storePath = Path.Combine(temporaryDirectory, "resource-space.tosumu");
    var sessionArguments = new
    {
        provider = "tosumu",
        store_path = storePath,
        store_id = "901",
        root_id = "902",
        root_folder_id = "903",
        case_policy = "sensitive",
    };
    var payload = Convert.ToBase64String(Encoding.UTF8.GetBytes("durable consumer contract"));
    try
    {
        await using (var first = new ResourceBridgeClient(bridgePath))
        {
            var created = await first.ExecuteAsync("session.create_or_open", sessionArguments);
            AssertEqual("created", created.GetProperty("outcome").GetString(), "Tosumu provider creates a durable session");
            await first.ExecuteAsync("resource.put", new
            {
                name = "durable.txt",
                bytes_base64 = payload,
                visibility = "hidden",
                media_type = "text/plain",
            });
        }

        await using (var second = new ResourceBridgeClient(bridgePath))
        {
            var reopened = await second.ExecuteAsync("session.create_or_open", sessionArguments);
            AssertEqual("opened_existing", reopened.GetProperty("outcome").GetString(), "fresh bridge process reopens Tosumu state");
            var hidden = await second.ExecuteAsync("resource.list", new { visibility = "hidden" });
            AssertEqual(1, hidden.GetProperty("resources").GetArrayLength(), "hidden navigation survives durable reopen");
            var fetched = await second.ExecuteAsync("resource.get", new { name = "durable.txt" });
            AssertEqual(payload, fetched.GetProperty("bytes_base64").GetString(), "exact resource bytes survive a fresh bridge process");
            var provider = await second.ExecuteAsync("provider.inspect");
            AssertEqual(true, provider.GetProperty("durable").GetBoolean(), "provider inspection labels Tosumu durability explicitly");
        }
    }
    finally
    {
        Directory.Delete(temporaryDirectory, recursive: true);
    }
}

static async Task WriteProviderOperationEvidenceAsync(string bridgePath)
{
    var temporaryDirectory = Path.Combine(Path.GetTempPath(), $"tokimu-resource-conformance-{Guid.NewGuid():N}");
    Directory.CreateDirectory(temporaryDirectory);
    try
    {
        var inMemory = await RunProviderOperationScenarioAsync(
            bridgePath,
            new { provider = "in_memory", store_id = "1101", root_id = "1102", root_folder_id = "1103" },
            reopenAfterMutation: false);
        var tosumu = await RunProviderOperationScenarioAsync(
            bridgePath,
            new
            {
                provider = "tosumu",
                store_path = Path.Combine(temporaryDirectory, "conformance.tosumu"),
                store_id = "1101",
                root_id = "1102",
                root_folder_id = "1103",
            },
            reopenAfterMutation: true);
        var durableEvidence = await CaptureTosumuDurableEvidenceAsync(
            bridgePath,
            new
            {
                provider = "tosumu",
                store_path = Path.Combine(temporaryDirectory, "conformance.tosumu"),
                store_id = "1101",
                root_id = "1102",
                root_folder_id = "1103",
            });

        var equal = JsonNode.DeepEquals(inMemory, tosumu);
        AssertTrue(equal, "in-memory and Tosumu providers retain the same operation observations");

        // This profile isolates Resource Space behavior from the broader
        // loader-oriented hello-resource-space report. Both providers run the
        // same semantic workflow; the durable reopen remains separate evidence.
        var artifact = new JsonObject
        {
            ["schema"] = 1,
            ["contract"] = "resource-space-provider-conformance-v1",
            ["profile"] = "provider-operation-fixture-v1",
            ["fixture"] = "folder-hidden-move-visible-retrieval",
            ["providers"] = new JsonObject
            {
                ["in_memory"] = inMemory,
                ["tosumu_reopened"] = tosumu,
            },
            ["comparison"] = new JsonObject
            {
                ["semantics_equal"] = equal,
                ["comparison_boundary"] = "summary, folder navigation, visibility filtering, move, metadata, and exact resource bytes",
            },
            ["durable_only"] = new JsonObject
            {
                ["fresh_bridge_reopen"] = true,
                ["provider"] = "tosumu",
                ["session"] = durableEvidence["session"]?.DeepClone(),
                ["provider_inspection"] = durableEvidence["provider_inspection"]?.DeepClone(),
            },
            ["expectations"] = new JsonObject
            {
                ["provider_boundary"] = "The providers expose equivalent public Resource Space observations without exposing backing collections, host paths, Tosumu keys, pages, WAL frames, or database records.",
                ["persistence_boundary"] = "Durable reopen and provider inspection are separately labeled evidence; they do not redefine the shared Resource Space semantics.",
                ["deferred"] = "Interrupted-write, corruption, transaction, and resource-limit evidence remain provider-specific work until Tosumu exposes bounded public observations for those cases.",
            },
        };
        var output = Path.Combine(
            FindRepositoryRoot(),
            "target",
            "resource-space-conformance",
            "dotnet-tosumu-resource-workbench",
            "provider-conformance-v1.json");
        Directory.CreateDirectory(Path.GetDirectoryName(output)!);
        await File.WriteAllTextAsync(
            output,
            artifact.ToJsonString(new JsonSerializerOptions { WriteIndented = true }) + Environment.NewLine);
        Console.WriteLine($"provider-conformance-artifact={output}");
    }
    finally
    {
        Directory.Delete(temporaryDirectory, recursive: true);
    }
}

static async Task WriteCompressionProviderEvidenceAsync(string bridgePath)
{
    var temporaryDirectory = Path.Combine(Path.GetTempPath(), $"tokimu-resource-compression-{Guid.NewGuid():N}");
    Directory.CreateDirectory(temporaryDirectory);
    try
    {
        var inMemory = await RunCompressionScenarioAsync(
            bridgePath,
            new { provider = "in_memory", store_id = "1201", root_id = "1202", root_folder_id = "1203" },
            reopenAfterMutation: false);
        var tosumu = await RunCompressionScenarioAsync(
            bridgePath,
            new
            {
                provider = "tosumu",
                store_path = Path.Combine(temporaryDirectory, "compression.tosumu"),
                store_id = "1201",
                root_id = "1202",
                root_folder_id = "1203",
            },
            reopenAfterMutation: true);
        var equal = JsonNode.DeepEquals(inMemory, tosumu);
        AssertTrue(equal, "in-memory and Tosumu-backed sessions retain identical compression observations");

        var artifact = new JsonObject
        {
            ["schema"] = 1,
            ["contract"] = "resource-space-provider-conformance-v1",
            ["profile"] = "compression-roundtrip-fixture-v1",
            ["fixture"] = "gzip-encode-decode-explicit-destinations",
            ["providers"] = new JsonObject
            {
                ["in_memory"] = inMemory,
                ["tosumu_reopened"] = tosumu,
            },
            ["comparison"] = new JsonObject
            {
                ["semantics_equal"] = equal,
                ["comparison_boundary"] = "explicit source and destination identity, retained source bytes, GZip observation, result fingerprint, collision outcome, and decoded resource bytes",
            },
            ["expectations"] = new JsonObject
            {
                ["resource_space_boundary"] = "Compression is an explicit bridge command. Ordinary reads remain byte-faithful and neither provider exposes backing-store details.",
                ["provider_boundary"] = "Tosumu durability is exercised by a fresh bridge process, while the compared observations remain Resource Space and compression-contract values.",
                ["scope"] = "The consumer-local Tosumu snapshot adapter is durable-host evidence, not an independent persistent Resource Space implementation.",
            },
        };
        var output = Path.Combine(
            FindRepositoryRoot(),
            "target",
            "resource-space-conformance",
            "dotnet-tosumu-resource-workbench",
            "compression-provider-conformance-v1.json");
        Directory.CreateDirectory(Path.GetDirectoryName(output)!);
        await File.WriteAllTextAsync(
            output,
            artifact.ToJsonString(new JsonSerializerOptions { WriteIndented = true }) + Environment.NewLine);
        Console.WriteLine($"compression-provider-conformance-artifact={output}");
    }
    finally
    {
        Directory.Delete(temporaryDirectory, recursive: true);
    }
}

static async Task<JsonNode> RunCompressionScenarioAsync(
    string bridgePath,
    object sessionArguments,
    bool reopenAfterMutation)
{
    var sourceBytes = Encoding.UTF8.GetBytes("Resource Space compression provider conformance fixture.");
    var source = Convert.ToBase64String(sourceBytes);
    var bridge = new ResourceBridgeClient(bridgePath);
    var bridgeDisposed = false;
    try
    {
        await bridge.ExecuteAsync("session.create_or_open", sessionArguments);
        await bridge.ExecuteAsync("resource.put", new
        {
            name = "report.txt",
            bytes_base64 = source,
            media_type = "text/plain",
        });
        var encoded = await bridge.ExecuteAsync("resource.transform_compression", new
        {
            source_name = "report.txt",
            destination_name = "report.txt.gz",
            operation = "encode",
            codec = "gzip",
            goal = "balanced",
            collision = "reject",
            media_type = "application/gzip",
        });
        var decoded = await bridge.ExecuteAsync("resource.transform_compression", new
        {
            source_name = "report.txt.gz",
            destination_name = "report-copy.txt",
            operation = "decode",
            codec = "gzip",
            collision = "reject",
            media_type = "text/plain",
        });

        if (reopenAfterMutation)
        {
            await bridge.DisposeAsync();
            bridgeDisposed = true;
            await using var reopened = new ResourceBridgeClient(bridgePath);
            await reopened.ExecuteAsync("session.create_or_open", sessionArguments);
            return await CaptureCompressionObservationAsync(reopened, encoded, decoded);
        }

        return await CaptureCompressionObservationAsync(bridge, encoded, decoded);
    }
    finally
    {
        if (!bridgeDisposed)
        {
            await bridge.DisposeAsync();
        }
    }
}

static async Task<JsonNode> CaptureCompressionObservationAsync(
    ResourceBridgeClient bridge,
    JsonElement encoded,
    JsonElement decoded)
{
    var source = await bridge.ExecuteAsync("resource.get", new { name = "report.txt" });
    var compressed = await bridge.ExecuteAsync("resource.get", new { name = "report.txt.gz" });
    var restored = await bridge.ExecuteAsync("resource.get", new { name = "report-copy.txt" });
    AssertEqual(
        source.GetProperty("bytes_base64").GetString(),
        restored.GetProperty("bytes_base64").GetString(),
        "compression fixture decode restores the original retained bytes");
    return new JsonObject
    {
        ["encoded"] = JsonNode.Parse(encoded.GetRawText()),
        ["decoded"] = JsonNode.Parse(decoded.GetRawText()),
        ["source"] = JsonNode.Parse(source.GetRawText()),
        ["compressed"] = JsonNode.Parse(compressed.GetRawText()),
        ["restored"] = JsonNode.Parse(restored.GetRawText()),
    };
}

static async Task WriteArchiveProviderEvidenceAsync(string bridgePath)
{
    var temporaryDirectory = Path.Combine(Path.GetTempPath(), $"tokimu-resource-archive-{Guid.NewGuid():N}");
    Directory.CreateDirectory(temporaryDirectory);
    try
    {
        var inMemory = await RunArchiveInspectionScenarioAsync(
            bridgePath,
            new { provider = "in_memory", store_id = "1301", root_id = "1302", root_folder_id = "1303" },
            reopenAfterMutation: false);
        var tosumu = await RunArchiveInspectionScenarioAsync(
            bridgePath,
            new
            {
                provider = "tosumu",
                store_path = Path.Combine(temporaryDirectory, "archive.tosumu"),
                store_id = "1301",
                root_id = "1302",
                root_folder_id = "1303",
            },
            reopenAfterMutation: true);
        var equal = JsonNode.DeepEquals(inMemory, tosumu);
        AssertTrue(equal, "in-memory and Tosumu-backed sessions retain identical archive inspection observations");

        var artifact = new JsonObject
        {
            ["schema"] = 1,
            ["contract"] = "resource-space-provider-conformance-v1",
            ["profile"] = "archive-inspection-and-import-fixture-v1",
            ["fixture"] = "zip-directory-and-single-file-static-bytes",
            ["providers"] = new JsonObject
            {
                ["in_memory"] = inMemory,
                ["tosumu_reopened"] = tosumu,
            },
            ["comparison"] = new JsonObject
            {
                ["semantics_equal"] = equal,
                ["comparison_boundary"] = "retained source bytes, source metadata, provider-neutral ZIP manifest entries, explicit imported folder hierarchy, and exact imported entry bytes after optional durable reopen",
            },
            ["expectations"] = new JsonObject
            {
                ["resource_space_boundary"] = "The desktop host contributes opaque fixture bytes only. Archive inspection and explicit subtree import run inside the Rust bridge and return no archive-library DTOs.",
                ["provider_boundary"] = "Tosumu durability is exercised by a fresh bridge process, while the compared observation remains Resource Space and archive-contract data.",
                ["scope"] = "The consumer-local Tosumu snapshot adapter is durable-host evidence, not an independent persistent Resource Space implementation.",
            },
        };
        var output = Path.Combine(
            FindRepositoryRoot(),
            "target",
            "resource-space-conformance",
            "dotnet-tosumu-resource-workbench",
            "archive-provider-conformance-v1.json");
        Directory.CreateDirectory(Path.GetDirectoryName(output)!);
        await File.WriteAllTextAsync(
            output,
            artifact.ToJsonString(new JsonSerializerOptions { WriteIndented = true }) + Environment.NewLine);
        Console.WriteLine($"archive-provider-conformance-artifact={output}");
    }
    finally
    {
        Directory.Delete(temporaryDirectory, recursive: true);
    }
}

static async Task<JsonNode> RunArchiveInspectionScenarioAsync(
    string bridgePath,
    object sessionArguments,
    bool reopenAfterMutation)
{
    // This fixed ZIP is opaque host input. The Rust bridge remains the only archive inspector.
    const string fixtureBase64 = "UEsDBBQAAAAAAMqVBF0AAAAAAAAAAAAAAAAFAAAAZG9jcy9QSwMEFAAAAAgAypUEXb8juOYsAAAAJAAAAA8AAABkb2NzL3JlYWRtZS50eHRKLErOyCxLVSgoyi/LTEktUkjOz0vLL8pNzEtOVUjLrCgpLUoFAAAA//8DAFBLAQIUABQAAAAAAMqVBF0AAAAAAAAAAAAAAAAFAAAAAAAAAAAAAAAAAAAAAABkb2NzL1BLAQIUABQAAAAIAMqVBF2/I7jmLAAAACQAAAAPAAAAAAAAAAAAAAAAACMAAABkb2NzL3JlYWRtZS50eHRQSwUGAAAAAAIAAgBwAAAAfAAAAAAA";
    var bridge = new ResourceBridgeClient(bridgePath);
    var bridgeDisposed = false;
    try
    {
        await bridge.ExecuteAsync("session.create_or_open", sessionArguments);
        await bridge.ExecuteAsync("resource.put", new
        {
            name = "fixture.zip",
            bytes_base64 = fixtureBase64,
            media_type = "application/zip",
        });

        if (reopenAfterMutation)
        {
            await bridge.DisposeAsync();
            bridgeDisposed = true;
            await using var reopened = new ResourceBridgeClient(bridgePath);
            await reopened.ExecuteAsync("session.create_or_open", sessionArguments);
            return await CaptureArchiveInspectionObservationAsync(reopened);
        }

        return await CaptureArchiveInspectionObservationAsync(bridge);
    }
    finally
    {
        if (!bridgeDisposed)
        {
            await bridge.DisposeAsync();
        }
    }
}

static async Task<JsonNode> CaptureArchiveInspectionObservationAsync(ResourceBridgeClient bridge)
{
    var source = await bridge.ExecuteAsync("resource.get", new { name = "fixture.zip" });
    var inspection = await bridge.ExecuteAsync("resource.inspect_archive", new
    {
        source_name = "fixture.zip",
        format = "zip",
    });
    var observation = inspection.GetProperty("observation");
    AssertEqual("zip", observation.GetProperty("format").GetString(), "archive inspection reports ZIP format");
    AssertEqual(2, observation.GetProperty("entries").GetArrayLength(), "archive inspection retains both directory and file entries");
    AssertEqual(
        "docs/readme.txt",
        observation.GetProperty("entries")[1].GetProperty("normalized_name").GetString(),
        "archive inspection preserves normalized entry identity");
    var imported = await bridge.ExecuteAsync("resource.import_archive_subtree", new
    {
        source_name = "fixture.zip",
        format = "zip",
        destination_root_name = "unpacked",
    });
    var importObservation = imported.GetProperty("observation");
    AssertEqual(2, importObservation.GetProperty("folders").GetInt32(), "archive import creates the explicit root and nested directory");
    AssertEqual(1, importObservation.GetProperty("resources").GetInt32(), "archive import materializes the regular-file entry");

    var rootFolders = await bridge.ExecuteAsync("folder.list");
    var unpackedId = rootFolders
        .GetProperty("folders")
        .EnumerateArray()
        .First(folder => folder.GetProperty("name").GetString() == "unpacked")
        .GetProperty("id")
        .GetString();
    AssertTrue(!string.IsNullOrWhiteSpace(unpackedId), "archive import returns a navigable destination root");
    var childFolders = await bridge.ExecuteAsync("folder.list", new { parent_folder_id = unpackedId });
    var docsId = childFolders.GetProperty("folders")[0].GetProperty("id").GetString();
    var importedResources = await bridge.ExecuteAsync("resource.list", new { parent_folder_id = docsId });
    AssertEqual(1, importedResources.GetProperty("resources").GetArrayLength(), "archive import retains one nested resource");
    AssertEqual("readme.txt", importedResources.GetProperty("resources")[0].GetProperty("name").GetString(), "archive import retains the leaf name");
    var importedBytes = await bridge.ExecuteAsync("resource.get", new { parent_folder_id = docsId, name = "readme.txt" });
    AssertEqual(
        Convert.ToBase64String(Encoding.UTF8.GetBytes("archive provider conformance fixture")),
        importedBytes.GetProperty("bytes_base64").GetString(),
        "archive import retains exact selected entry bytes");
    return new JsonObject
    {
        ["source"] = JsonNode.Parse(source.GetRawText()),
        ["inspection"] = JsonNode.Parse(inspection.GetRawText()),
        ["import"] = JsonNode.Parse(imported.GetRawText()),
        ["imported_root_folders"] = JsonNode.Parse(rootFolders.GetRawText()),
        ["imported_child_folders"] = JsonNode.Parse(childFolders.GetRawText()),
        ["imported_resources"] = JsonNode.Parse(importedResources.GetRawText()),
        ["imported_bytes"] = JsonNode.Parse(importedBytes.GetRawText()),
    };
}

static async Task<JsonObject> CaptureTosumuDurableEvidenceAsync(
    string bridgePath,
    object sessionArguments)
{
    await using var bridge = new ResourceBridgeClient(bridgePath);
    var session = await bridge.ExecuteAsync("session.create_or_open", sessionArguments);
    AssertEqual("opened_existing", session.GetProperty("outcome").GetString(), "durable evidence uses a fresh reopened session");
    var inspection = await bridge.ExecuteAsync("provider.inspect");
    AssertEqual(true, inspection.GetProperty("durable").GetBoolean(), "durable evidence identifies the Tosumu provider");
    return new JsonObject
    {
        ["session"] = JsonNode.Parse(session.GetRawText()),
        ["provider_inspection"] = JsonNode.Parse(inspection.GetRawText()),
    };
}

static async Task<JsonNode> RunProviderOperationScenarioAsync(
    string bridgePath,
    object sessionArguments,
    bool reopenAfterMutation)
{
    var payload = Convert.ToBase64String(Encoding.UTF8.GetBytes("provider operation fixture"));
    var bridge = new ResourceBridgeClient(bridgePath);
    var bridgeDisposed = false;
    try
    {
        await bridge.ExecuteAsync("session.create_or_open", sessionArguments);
        var folder = await bridge.ExecuteAsync("folder.create", new { name = "notes" });
        var folderId = folder.GetProperty("folder").GetProperty("id").GetString();
        await bridge.ExecuteAsync("resource.put", new
        {
            parent_folder_id = folderId,
            name = "draft.txt",
            bytes_base64 = payload,
            visibility = "hidden",
            media_type = "text/plain",
        });
        var hidden = await bridge.ExecuteAsync("resource.list", new
        {
            parent_folder_id = folderId,
            visibility = "hidden",
        });
        AssertEqual(1, hidden.GetProperty("resources").GetArrayLength(), "hidden fixture remains discoverable");
        var hiddenBeforeMove = JsonNode.Parse(hidden.GetRawText())
            ?? throw new InvalidOperationException("hidden fixture observation must be valid JSON");
        await bridge.ExecuteAsync("resource.move", new
        {
            source_parent_folder_id = folderId,
            source_name = "draft.txt",
            destination_name = "published.txt",
        });
        await bridge.ExecuteAsync("resource.set_visibility", new
        {
            name = "published.txt",
            visibility = "visible",
        });

        if (reopenAfterMutation)
        {
            await bridge.DisposeAsync();
            bridgeDisposed = true;
            await using var reopened = new ResourceBridgeClient(bridgePath);
            await reopened.ExecuteAsync("session.create_or_open", sessionArguments);
            return await CaptureProviderOperationObservationAsync(reopened, hiddenBeforeMove);
        }

        return await CaptureProviderOperationObservationAsync(bridge, hiddenBeforeMove);
    }
    finally
    {
        if (!bridgeDisposed)
        {
            await bridge.DisposeAsync();
        }
    }
}

static async Task<JsonNode> CaptureProviderOperationObservationAsync(
    ResourceBridgeClient bridge,
    JsonNode hiddenBeforeMove)
{
    var summary = await bridge.ExecuteAsync("observation.summary");
    var folders = await bridge.ExecuteAsync("folder.list");
    var visible = await bridge.ExecuteAsync("resource.list", new { visibility = "visible" });
    var all = await bridge.ExecuteAsync("resource.list", new { visibility = "all" });
    var fetched = await bridge.ExecuteAsync("resource.get", new { name = "published.txt" });
    return new JsonObject
    {
        ["summary"] = JsonNode.Parse(summary.GetRawText()),
        ["folders"] = JsonNode.Parse(folders.GetRawText()),
        ["hidden_before_move"] = hiddenBeforeMove,
        ["visible"] = JsonNode.Parse(visible.GetRawText()),
        ["all"] = JsonNode.Parse(all.GetRawText()),
        ["published"] = JsonNode.Parse(fetched.GetRawText()),
    };
}

static string FindRepositoryRoot()
{
    for (var current = new DirectoryInfo(AppContext.BaseDirectory); current is not null; current = current.Parent)
    {
        if (Directory.Exists(Path.Combine(current.FullName, ".git")))
        {
            return current.FullName;
        }
    }

    throw new DirectoryNotFoundException("Could not locate the Tokimu repository root for evidence output.");
}

static string WriteCommand(string directory, string name, string body)
{
    var path = Path.Combine(directory, $"{name}.cmd");
    File.WriteAllText(path, "@echo off\r\n" + body + "\r\n", Encoding.ASCII);
    return path;
}

static async Task<TException> AssertThrowsAsync<TException>(Func<Task<JsonElement>> operation, string description)
    where TException : Exception
{
    try
    {
        await operation();
    }
    catch (TException exception)
    {
        return exception;
    }

    throw new InvalidOperationException($"Assertion failed: {description}. Expected {typeof(TException).Name}.");
}
