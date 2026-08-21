# Host Shortcuts Are Input-Boundary Constraints

An application binding is not usable merely because every key in the chord can
be observed individually. The host platform may reserve the combined chord and
act before, instead of, or in addition to the application.

Tokimu's browser Doom workbench exposed the failure directly:

```text
application controls
    Ctrl = descend
    W    = move forward

combined movement
    Ctrl + W

Edge host meaning
    close the current window
```

The resulting orderly browser disappearance initially resembled a renderer or
WebGPU failure. External terminal observation found no device-loss, OOM, crash,
or fatal record. Input history then correlated the exit with forward-plus-
descend movement, and changing descend to `C` removed the collision.

## General Lesson

Input availability is platform-conditioned:

```text
physical keys
    -> host shortcut policy
    -> events actually delivered to Tokimu
    -> normalized Tokimu input
    -> application binding
```

A normalized input model cannot recover a chord that the browser, window
manager, operating system, accessibility service, or embedding host consumes.
Likewise, receiving the component key-down events does not prove that the host
will permit their combination without side effects.

Do not diagnose disappearance from silence. Preserve the terminal outcome as
unknown until an observer outside the affected page/window/process supplies
evidence. A host shortcut may terminate the subject cleanly while producing no
application error at all.

## Binding Review

For interactive corpus work and application defaults:

1. Test chords, not only individual keys.
2. Include simultaneous movement and modifier combinations in the input matrix.
3. Check browser, desktop, window-manager, and accessibility reservations for
   every supported target.
4. Prefer unmodified gameplay/navigation defaults where practical.
5. Supply a non-conflicting fallback and allow application-level remapping.
6. Treat `preventDefault` or keyboard capture as target-specific evidence, not
   a portable guarantee.
7. Keep an external observer active when a conflicting chord could close the
   page, window, process, or another diagnostic domain.

## Possible Platform Contract

Repeated cross-target pressure may justify a Tokimu-owned, platform-reported
binding-constraint contract. Such a contract could distinguish:

```text
unavailable
    host always owns the chord

conditionally interceptable
    application may receive it only in a declared focus/fullscreen context

delivered with host side effect
    application receives input but the host also acts

available
    no known host conflict on the tested platform profile

unknown
    no portable claim has been established
```

The platform adapter would report target constraints; the application would
still own action meaning, default bindings, fallback choice, and remapping
policy. `tokimu-core` should not become a catalog of browser or operating-system
shortcuts.

This vocabulary is a study direction, not an admitted stable API. It should be
promoted only if multiple callers need more than corpus-local conflict checks.

## Evidence

- [ADR-0017 terminal failure and host crash conformance](../ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md)
- [AR-0024 renderer failure observation boundary](../Architectural%20Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md)
- [Renderer resource-lifetime baseline evidence](../Plans/Renderer-Reliability/Evidence/renderer-scene-resource-lifetime-baseline-and-inventory-evidence.md)
- [Doom WAD checklist](../Plans/DOOM/DOOM%20WAD%20Checklist.md)
