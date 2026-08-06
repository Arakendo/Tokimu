using System.IO;

namespace Tokimu.ResourceWorkbench;

internal sealed record RepositoryLayout(
    bool TokimuCheckoutFound,
    bool TosumuSubmoduleFound,
    string? BridgePath,
    string Status);

internal static class RepositoryLayoutDiscovery
{
    public static RepositoryLayout Discover()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);

        while (directory is not null)
        {
            var gitDirectory = Path.Combine(directory.FullName, ".git");
            var submodule = Path.Combine(directory.FullName, "third-party", "tosumu");
            if (Directory.Exists(gitDirectory) || File.Exists(gitDirectory))
            {
                var bridge = Path.Combine(directory.FullName, "target", "debug", "tokimu-resource-workbench-bridge.exe");
                return Directory.Exists(submodule)
                    ? new RepositoryLayout(true, true, File.Exists(bridge) ? bridge : null, "Tokimu checkout and pinned Tosumu submodule discovered.")
                    : new RepositoryLayout(true, false, File.Exists(bridge) ? bridge : null, "Tokimu checkout discovered; pinned third-party/tosumu submodule is missing.");
            }

            directory = directory.Parent;
        }

        return new RepositoryLayout(false, false, null, "Tokimu checkout could not be discovered from the host process.");
    }
}
