using System.Diagnostics;
using System.Text.Json;

namespace Tokimu.ResourceWorkbench.Bridge;

/// <summary>
/// Bounded JSON-lines client for the consumer-local Tokimu bridge.
/// It intentionally knows the bridge envelope, not Tosumu commands or storage.
/// </summary>
public sealed class ResourceBridgeClient : IAsyncDisposable
{
    private readonly Process process;
    private readonly SemaphoreSlim requestGate = new(1, 1);
    private int requestSequence;
    private bool disposed;

    public ResourceBridgeClient(string executablePath)
    {
        if (string.IsNullOrWhiteSpace(executablePath))
        {
            throw new ArgumentException("A bridge executable path is required.", nameof(executablePath));
        }

        var startInfo = new ProcessStartInfo(executablePath)
        {
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true,
        };
        process = Process.Start(startInfo)
            ?? throw new ResourceBridgeTransportException("Bridge process could not be started.");
    }

    public async Task<JsonElement> ExecuteAsync(
        string command,
        object? arguments = null,
        CancellationToken cancellationToken = default)
    {
        await requestGate.WaitAsync(cancellationToken);
        try
        {
            ThrowIfExited();
            var requestId = $"dotnet-{Interlocked.Increment(ref requestSequence)}";
            var request = new
            {
                schema = 1,
                request_id = requestId,
                command,
                arguments = arguments ?? new { },
            };

            await process.StandardInput.WriteLineAsync(JsonSerializer.Serialize(request));
            await process.StandardInput.FlushAsync(cancellationToken);
            var line = await process.StandardOutput.ReadLineAsync(cancellationToken);
            if (string.IsNullOrWhiteSpace(line))
            {
                var stderr = await process.StandardError.ReadToEndAsync(cancellationToken);
                throw new ResourceBridgeTransportException(
                    $"Bridge closed without a response. stderr: {stderr.Trim()}");
            }

            try
            {
                using var document = JsonDocument.Parse(line);
                var root = document.RootElement;
                if (!root.TryGetProperty("request_id", out var returnedRequestId)
                    || returnedRequestId.GetString() != requestId)
                {
                    throw new ResourceBridgeProtocolException("Bridge response request_id did not match the request.");
                }
                if (!root.TryGetProperty("ok", out var ok) || ok.ValueKind is not JsonValueKind.True and not JsonValueKind.False)
                {
                    throw new ResourceBridgeProtocolException("Bridge response did not contain a boolean ok field.");
                }
                if (!ok.GetBoolean())
                {
                    var kind = root.TryGetProperty("error", out var error)
                        && error.TryGetProperty("kind", out var errorKind)
                        ? errorKind.GetString()
                        : "unknown";
                    var message = root.TryGetProperty("error", out error)
                        && error.TryGetProperty("message", out var errorMessage)
                        ? errorMessage.GetString()
                        : "Bridge rejected the command.";
                    throw new ResourceBridgeCommandException(kind ?? "unknown", message ?? "Bridge rejected the command.");
                }
                if (!root.TryGetProperty("result", out var result))
                {
                    throw new ResourceBridgeProtocolException("Successful bridge response did not contain result.");
                }
                return result.Clone();
            }
            catch (JsonException exception)
            {
                throw new ResourceBridgeProtocolException($"Bridge emitted malformed JSON: {exception.Message}", exception);
            }
        }
        finally
        {
            requestGate.Release();
        }
    }

    public async ValueTask DisposeAsync()
    {
        await requestGate.WaitAsync();
        try
        {
            if (disposed) return;
            disposed = true;
            process.StandardInput.Close();
            if (!process.HasExited)
            {
                using var cancellation = new CancellationTokenSource(TimeSpan.FromSeconds(2));
                try
                {
                    await process.WaitForExitAsync(cancellation.Token);
                }
                catch (OperationCanceledException)
                {
                    process.Kill(entireProcessTree: true);
                }
            }
            process.Dispose();
        }
        finally
        {
            requestGate.Release();
        }
    }

    private void ThrowIfExited()
    {
        if (disposed)
        {
            throw new ObjectDisposedException(nameof(ResourceBridgeClient));
        }
        if (process.HasExited)
        {
            throw new ResourceBridgeTransportException(
                $"Bridge process exited with code {process.ExitCode} before handling the request.");
        }
    }
}

public class ResourceBridgeTransportException : Exception
{
    public ResourceBridgeTransportException(string message) : base(message) { }
    public ResourceBridgeTransportException(string message, Exception innerException) : base(message, innerException) { }
}

public sealed class ResourceBridgeProtocolException : ResourceBridgeTransportException
{
    public ResourceBridgeProtocolException(string message) : base(message) { }
    public ResourceBridgeProtocolException(string message, Exception innerException) : base(message, innerException) { }
}

public sealed class ResourceBridgeCommandException : Exception
{
    public ResourceBridgeCommandException(string kind, string message) : base(message)
    {
        Kind = kind;
    }

    public string Kind { get; }
}
