# FBX Corpus

## Purpose

`fbx-corpus` makes FBX source structure observable without admitting FBX as a
Tokimu engine capability.

The first profile owns only bounded binary decoding:

```text
binary FBX bytes
    -> source records and properties
    -> deterministic structural artifact
```

It does not resolve objects or connections, lower meshes, evaluate transforms,
or submit renderer work. Those stages remain separate corpus slices so a
failure can be assigned to the first boundary whose evidence diverges.

## Ownership

- This crate owns corpus-only FBX syntax and diagnostics.
- Tokimu engine crates do not depend on this crate.
- FBX source records do not define Tokimu model semantics.
- Rendering is outside this crate.
