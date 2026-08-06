# Evidence

This folder will retain manual Windows screenshots and bounded bridge/provider
artifacts after their owning slices exist. Generated `bin`, `obj`, package
caches, and copied executables are not evidence and must not be committed.

The headless check currently writes its deterministic provider-only comparison
artifact under
`target/resource-space-conformance/dotnet-tosumu-resource-workbench/`. It is
kept outside this source folder because it is generated evidence, not a checked
in fixture. Native-window screenshots remain separately labeled manual
evidence.
