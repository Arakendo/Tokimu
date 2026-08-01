# UI Resource Workbench

## Purpose

`ui-resource-workbench` is an application-shaped consumer corpus for the
provider-neutral contracts in `ui-tools`.

It pressures a denser workflow than the settings workbench:

- filterable, selectable application records;
- stable interaction identity across filtering and responsive layouts;
- editable detail state with explicit apply and revert commands;
- command availability derived from application state;
- destructive action confirmation through a modal scope;
- deterministic semantic-tree lowering.

## Ownership

The application model owns resource identity, filtering, selection, drafts,
validation, deletion policy, and status messages. `ui-tools` owns layout,
semantic nodes, interaction routing, modal confinement, and renderer-neutral
draw lowering. The native host owns platform events and GPU submission.

The corpus must not move resource meaning into `ui-tools`, and its native host
must not reinterpret application commands.

## Composition Claim

The same application state and semantic scene builder must support both a wide
inspector layout and a compact stacked layout. Pointer, focus, and text input
must address the same stable node identities in both arrangements.

The model and scene are intentionally independent of the native renderer so a
later website Lab consumer can adapt the same contract without duplicating
resource semantics in TypeScript.

## Controls

- Click a resource row to select it.
- Edit the filter, name, and notes fields with normal text input.
- Activate visibility and hotspot controls to toggle their draft values.
- Apply or revert a dirty draft.
- Delete opens a confirmation modal; Escape or Cancel dismisses it.
- Up and Down move focus. Enter or Space activates the focused command.

## Acceptance Criteria

- Wide and compact scenes resolve without layout diagnostics.
- Filtering never changes a resource's semantic row identity.
- Clean apply and revert commands are not interactive.
- A modal excludes background controls from pointer and focus routing.
- Deletion cannot occur without confirmation and cannot remove the final item.
- Repeated lowering produces the same structural fingerprint.
