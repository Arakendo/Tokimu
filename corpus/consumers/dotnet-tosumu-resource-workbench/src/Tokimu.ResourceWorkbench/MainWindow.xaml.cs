using System.IO;
using System.Text;
using System.Text.Json;
using System.Windows;
using Microsoft.Win32;
using Tokimu.ResourceWorkbench.Bridge;

namespace Tokimu.ResourceWorkbench;

public partial class MainWindow : Window
{
    private readonly RepositoryLayout layout;
    private readonly SemaphoreSlim workflowGate = new(1, 1);
    private ResourceBridgeClient? bridge;
    private string? durableStorePath;
    private string? rootFolderId;
    private string? currentFolderId;
    private string visibilityFilter = "all";
    private bool refreshing;

    public MainWindow()
    {
        InitializeComponent();
        layout = RepositoryLayoutDiscovery.Discover();
        RepositoryStatus.Text = layout.Status;
        BridgeStatus.Text = layout.BridgePath is null
            ? "Build the bounded bridge first: cargo build -p tokimu-resource-workbench-bridge."
            : "Bridge available. Open an in-memory or Tosumu-backed session to begin the consumer workflow.";
        HostSessionText.Text = "No host session is active.";
        CurrentFolderText.Text = "Open a session to browse its root folder.";
        ResourceMetadataText.Text = "Select a resource to inspect provider-neutral metadata.";
        Closed += async (_, _) =>
        {
            if (bridge is not null)
            {
                await bridge.DisposeAsync();
            }
        };
    }

    private async void OpenSession_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(() => OpenSessionAsync(
            new
            {
                provider = "in_memory",
                store_id = "2001",
                root_id = "2002",
                root_folder_id = "2003",
                display_name = "Desktop Resource Workbench",
                case_policy = "sensitive",
            },
            "In-memory Tokimu Resource Space session is active.",
            "Host session: in-memory. No durable location is selected."));
    }

    private async void OpenTosumuSession_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(async () =>
        {
            durableStorePath = GetDurableStorePath();
            await OpenSessionAsync(
            new
            {
                provider = "tosumu",
                store_path = durableStorePath,
                store_id = "2001",
                root_id = "2002",
                root_folder_id = "2003",
                display_name = "Desktop Resource Workbench",
                case_policy = "sensitive",
            },
                "Tosumu-backed Tokimu Resource Space session is active.",
                $"Host-selected durable store: {durableStorePath}{Environment.NewLine}The path is host configuration and is not included in Tokimu Resource Space or provider observations.");
        });
    }

    private async void CreateFolder_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(async () =>
        {
            if (!EnsureBridge()) return;
            try
            {
                await bridge!.ExecuteAsync("folder.create", new { parent_folder_id = currentFolderId, name = "notes" });
                BridgeStatus.Text = "Created the notes folder through the Tokimu bridge.";
                await RefreshObservationsAsync();
            }
            catch (Exception exception)
            {
                BridgeStatus.Text = $"Folder creation failed: {exception.Message}";
            }
        });
    }

    private async void PutSample_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(async () =>
        {
            if (!EnsureBridge()) return;
            try
            {
                await bridge!.ExecuteAsync("resource.put", new
                {
                    parent_folder_id = currentFolderId,
                    name = "welcome.txt",
                    bytes_base64 = Convert.ToBase64String(Encoding.UTF8.GetBytes("Tokimu Resource Space through .NET.")),
                    media_type = "text/plain",
                    visibility = "visible",
                });
                BridgeStatus.Text = "Added a visible text resource through the Tokimu bridge.";
                await RefreshObservationsAsync();
            }
            catch (Exception exception)
            {
                BridgeStatus.Text = $"Resource write failed: {exception.Message}";
            }
        });
    }

    private async void ImportFile_Click(object sender, RoutedEventArgs e)
    {
        if (!EnsureBridge() || string.IsNullOrWhiteSpace(currentFolderId)) return;

        var dialog = new OpenFileDialog
        {
            Title = "Import a resource into the current Tokimu folder",
            CheckFileExists = true,
            Multiselect = false,
        };
        if (dialog.ShowDialog(this) != true) return;

        await RunWorkflowAsync(async () =>
        {
            if (!EnsureBridge() || string.IsNullOrWhiteSpace(currentFolderId)) return;
            try
            {
                var bytes = await File.ReadAllBytesAsync(dialog.FileName);
                await bridge!.ExecuteAsync("resource.put", new
                {
                    parent_folder_id = currentFolderId,
                    name = Path.GetFileName(dialog.FileName),
                    bytes_base64 = Convert.ToBase64String(bytes),
                    media_type = GuessMediaType(dialog.FileName),
                    visibility = "visible",
                });
                BridgeStatus.Text = $"Imported {Path.GetFileName(dialog.FileName)} through the Tokimu bridge.";
                await RefreshObservationsAsync();
            }
            catch (Exception exception)
            {
                BridgeStatus.Text = $"File import failed: {exception.Message}";
            }
        });
    }

    private async void ToggleVisibility_Click(object sender, RoutedEventArgs e)
    {
        if (!EnsureBridge() || ResourceList.SelectedItem is not ResourceItem resource) return;

        await RunWorkflowAsync(async () =>
        {
            if (!EnsureBridge()) return;
            var visibility = resource.Visibility == "hidden" ? "visible" : "hidden";
            try
            {
                await bridge!.ExecuteAsync("resource.set_visibility", new
                {
                    parent_folder_id = resource.ParentFolderId,
                    name = resource.Name,
                    visibility,
                });
                BridgeStatus.Text = $"Set {resource.Name} visibility to {visibility} through the Tokimu bridge.";
                await RefreshObservationsAsync();
            }
            catch (Exception exception)
            {
                BridgeStatus.Text = $"Resource visibility change failed: {exception.Message}";
            }
        });
    }

    private async void Refresh_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(async () =>
        {
            if (EnsureBridge()) await RefreshObservationsAsync();
        });
    }

    private async void NavigateRoot_Click(object sender, RoutedEventArgs e)
    {
        await RunWorkflowAsync(async () =>
        {
            if (string.IsNullOrWhiteSpace(rootFolderId) || !EnsureBridge()) return;
            currentFolderId = rootFolderId;
            await RefreshObservationsAsync();
        });
    }

    private async void VisibilityFilter_SelectionChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
    {
        if (VisibilityFilter.SelectedItem is not System.Windows.Controls.ComboBoxItem item || item.Tag is not string filter) return;
        visibilityFilter = filter;
        if (refreshing) return;
        await RunWorkflowAsync(async () =>
        {
            if (bridge is not null) await RefreshObservationsAsync();
        });
    }

    private async void FolderList_SelectionChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
    {
        if (refreshing || FolderList.SelectedItem is not FolderItem folder) return;
        await RunWorkflowAsync(async () =>
        {
            if (!EnsureBridge() || folder.Id == currentFolderId) return;
            currentFolderId = folder.Id;
            await RefreshObservationsAsync();
        });
    }

    private void ResourceList_SelectionChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
    {
        if (refreshing) return;
        if (ResourceList.SelectedItem is ResourceItem resource)
        {
            ResourceMetadataText.Text = Format("Selected Resource", resource.Observation);
            ToggleVisibilityButton.IsEnabled = true;
            ToggleVisibilityButton.Content = resource.Visibility == "hidden"
                ? "Make selected resource visible"
                : "Hide selected resource";
            return;
        }

        ResourceMetadataText.Text = "Select a resource to inspect provider-neutral metadata.";
        ToggleVisibilityButton.IsEnabled = false;
        ToggleVisibilityButton.Content = "Toggle selected visibility";
    }

    private bool EnsureBridge()
    {
        if (bridge is not null) return true;
        BridgeStatus.Text = "Open a Tokimu session before using this workflow.";
        return false;
    }

    private async Task RunWorkflowAsync(Func<Task> workflow)
    {
        await workflowGate.WaitAsync();
        try
        {
            await workflow();
        }
        catch (Exception exception)
        {
            BridgeStatus.Text = $"Tokimu bridge workflow failed: {exception.Message}";
        }
        finally
        {
            workflowGate.Release();
        }
    }

    private async Task ReplaceBridgeAsync()
    {
        if (bridge is not null)
        {
            await bridge.DisposeAsync();
        }
        bridge = new ResourceBridgeClient(layout.BridgePath!);
        rootFolderId = null;
        currentFolderId = null;
    }

    private async Task OpenSessionAsync(object arguments, string activeMessage, string hostSessionDetails)
    {
        if (layout.BridgePath is null)
        {
            BridgeStatus.Text = "The bridge executable is unavailable. Build it from the Tokimu checkout first.";
            return;
        }

        await ReplaceBridgeAsync();
        try
        {
            var session = await bridge!.ExecuteAsync("session.create_or_open", arguments);
            var outcome = session.TryGetProperty("outcome", out var value) ? value.GetString() : null;
            rootFolderId = session.GetProperty("root_folder_id").GetString();
            currentFolderId = rootFolderId;
            BridgeStatus.Text = outcome is null ? activeMessage : $"{activeMessage} Session outcome: {outcome}.";
            HostSessionText.Text = hostSessionDetails;
            await RefreshObservationsAsync();
        }
        catch (Exception exception)
        {
            BridgeStatus.Text = $"Bridge session failed: {exception.Message}";
            HostSessionText.Text = "No host session is active.";
            CurrentFolderText.Text = "No folder is selected.";
        }
    }

    private static string GetDurableStorePath()
    {
        var directory = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "Tokimu",
            "ResourceWorkbench");
        Directory.CreateDirectory(directory);
        return Path.Combine(directory, "resource-space.tosumu");
    }

    private async Task RefreshObservationsAsync()
    {
        var activeBridge = bridge ?? throw new InvalidOperationException("A Resource Space session is required.");
        var summary = await activeBridge.ExecuteAsync("observation.summary");
        var folders = await activeBridge.ExecuteAsync("folder.list", new { parent_folder_id = currentFolderId, visibility = "all" });
        var resources = await activeBridge.ExecuteAsync("resource.list", new { parent_folder_id = currentFolderId, visibility = visibilityFilter });
        var provider = await activeBridge.ExecuteAsync("provider.inspect");
        if (!ReferenceEquals(activeBridge, bridge)) return;

        refreshing = true;
        try
        {
            ObservationText.Text = Format("Summary", summary) + Environment.NewLine + Environment.NewLine
                + Format("Current folder contents", new { folder_id = currentFolderId });
            CurrentFolderText.Text = currentFolderId is null ? "No folder is selected." : $"Folder id: {currentFolderId}";
            FolderList.ItemsSource = folders.GetProperty("folders")
                .EnumerateArray()
                .Select(folder => new FolderItem(
                    folder.GetProperty("id").GetString()!,
                    $"{folder.GetProperty("name").GetString() ?? "(root)"} [{folder.GetProperty("visibility").GetString()}]"))
                .ToArray();
            ResourceList.ItemsSource = resources.GetProperty("resources")
                .EnumerateArray()
                .Select(resource => new ResourceItem(
                    resource.GetProperty("name").GetString()!,
                    resource.GetProperty("parent_folder_id").GetString()!,
                    resource.GetProperty("visibility").GetString()!,
                    $"{resource.GetProperty("name").GetString()} ({resource.GetProperty("byte_length").GetUInt64()} bytes, {resource.GetProperty("visibility").GetString()})",
                    resource.Clone()))
                .ToArray();
            ProviderText.Text = Format("Provider-owned evidence", provider);
            ResourceMetadataText.Text = "Select a resource to inspect provider-neutral metadata.";
            ToggleVisibilityButton.IsEnabled = false;
            ToggleVisibilityButton.Content = "Toggle selected visibility";
        }
        finally
        {
            refreshing = false;
        }
    }

    private static string GuessMediaType(string path) => Path.GetExtension(path).ToLowerInvariant() switch
    {
        ".txt" or ".md" or ".json" or ".xml" or ".svg" => "text/plain",
        ".png" => "image/png",
        ".jpg" or ".jpeg" => "image/jpeg",
        ".bmp" => "image/bmp",
        _ => "application/octet-stream",
    };

    private static string Format(string label, JsonElement value) =>
        $"{label}{Environment.NewLine}{JsonSerializer.Serialize(value, new JsonSerializerOptions { WriteIndented = true })}";

    private static string Format(string label, object value) =>
        $"{label}{Environment.NewLine}{JsonSerializer.Serialize(value, new JsonSerializerOptions { WriteIndented = true })}";

    private sealed record FolderItem(string Id, string Label);
    private sealed record ResourceItem(string Name, string ParentFolderId, string Visibility, string Label, JsonElement Observation);
}
