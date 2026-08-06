# Hello Archive

## Purpose

Exercise bounded archive inspection, selected-entry reads, and deterministic
archive writing through the provider-neutral archive contract, then compose
the read semantics with an explicit Resource Space selected-entry copy. This
corpus does not extract to the host filesystem or make archives implicit
Resource Space folders.

## Primary Proof

```text
first-party ZIP fixture bytes
    -> bounded manifest inspection
    -> normalized entry observations
    -> selected regular-file read
    -> explicit Resource Space destination
    -> byte-identical payload + structural report

ordered provider-neutral entries
    -> fixed ZIP metadata and explicit compression
    -> bounded archive bytes
    -> byte-identical rebuild
    -> inspect + selected read round trip
```

The read fixture was generated once with `.NET ZipArchive` and is retained
as an immutable byte array. Fixture construction is not part of the application
contract: the executable consumes it only through `ArchiveProvider`. The
fixture contains one directory and two stored regular files.

The corpus also applies an archive-input budget smaller than the fixture and
requires an explicit `ArchiveLimitExceeded` result. It writes schema-2
`target/hello-archive/report.json` as structural evidence. Schema 2 adds the
bounded 7z compatibility observation without changing the earlier ZIP, TAR,
or Resource Space observations.

The write proof creates the same ordered semantic entry list twice and
requires byte-identical ZIP output. It then inspects and reads that output
through `ArchiveProvider`; the writer does not receive a filesystem path,
clock, or host metadata.

The corpus also creates a fresh 7z archive from the same logical file tree.
This compatibility proof requires provider-selected 7z compression, a
byte-identical rebuild in the current provider, manifest agreement with the
ZIP representation, and a selected-entry byte round trip. It does not claim
7z append, update, password, multi-volume, or metadata-preserving edit
semantics. That agreement compares portable logical entry kind, normalized
tree name, and uncompressed byte length: container-level compression metadata,
CRC availability, and directory trailing-slash spelling remain provider
observations.

The Resource Space proof retains the original archive bytes unchanged,
inspects them through `resource-space-archive`, and copies one caller-selected
entry to one caller-selected logical name. No filename or media type triggers
the operation.

## Non-Goals

- Host filesystem extraction or path-owned archive creation.
- Whole-tree Resource Space import, export, or archive-backed views.
- TAR.GZ, encryption, links, or writable archive mutation.
- 7z append, update, password, multi-volume, or metadata-preserving edits.
- Treating entry names as trusted logical resource addresses.
