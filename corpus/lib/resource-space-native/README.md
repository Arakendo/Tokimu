# Resource Space Native Adapter

`resource-space-native` is an incubating host-filesystem adapter for the
provider-neutral `resource-space` contract.

It deliberately owns native paths, directory traversal, hidden-entry policy,
and containment-checked export. It does **not** add filesystem concepts to
`resource-space`, does not expose host paths through resource keys, and does
not choose an application's store or root identity.

Import always targets a caller-created logical folder. Export always requires
a caller-approved native root and verifies each output parent remains beneath
that root.
