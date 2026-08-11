# Camera Direction Conformance Manifest

| Field | Retained value |
| --- | --- |
| Review pressure | AR-0028 Slice 3 |
| Scope | Non-Doom camera basis, deterministic command replay, and live input-policy evidence |
| Status | Incubating corpus vocabulary; no public camera/input contract admitted |
| Initial position | `[0,0,-6]` |
| Initial yaw / pitch | `0 / 0` radians |
| Initial forward | `+Z` |
| Initial right | `-X` |
| Initial up | `+Y` |
| Pitch limit | `±0.7` radians |

## Basis Construction

```text
forward = normalize([
    sin(yaw) * cos(pitch),
    sin(pitch),
    cos(yaw) * cos(pitch),
])

flat_forward = normalize([forward.x, 0, forward.z])
right = normalize(flat_forward × +Y)
up = normalize(right × forward)
```

Consequently, positive mathematical yaw from the initial `+Z` heading turns
toward `+X`, while initial screen-right is `-X`. The fixture never calls
positive yaw a right turn.

## Evidence Layers

| Layer | Native fixture evidence |
| --- | --- |
| Physical mechanism | Winit raw mouse motion, mouse button, and key events adapted by `tokimu-platform` |
| Normalized observation | `PlatformInputEvent::MouseMotion { delta_x, delta_y }` retained unchanged as `PointerMotionObservation` |
| Interaction policy | Corpus-local first-person policy maps `+delta_x` to negative yaw and `+delta_y` to negative pitch |
| Camera convention | `CameraConformancePose` applies semantic commands and derives the declared orthonormal basis |

Free pointer motion updates the retained physical observation but does not
produce a camera command. Clicking captures the pointer. Only captured raw
motion passes through the first-person policy. Escape releases capture.

## Deterministic Command Replay

The shared structural suite covers:

- positive yaw and negative-yaw screen-right behavior;
- positive/negative pitch and bounded pitch;
- forward and backward movement;
- positive/negative local-right strafe;
- positive/negative vertical movement;
- return to the initial pose after inverse movement commands;
- orthonormal forward/up/right results;
- pointer observation retained separately from mapped commands.

## Native Live Controls

| Input | Corpus-local policy |
| --- | --- |
| click | capture pointer |
| Escape | release pointer and clear held movement |
| W / S | positive / negative ground-plane forward |
| D / A | positive / negative local right |
| E / Q | positive / negative world `+Y` |
| Arrow left / right | positive / negative mathematical yaw |
| Arrow up / down | positive / negative pitch |
| captured raw mouse | first-person look policy |

The window title presents capture state, position, forward/up/right, last raw
pointer observation, and last mapped commands. Startup diagnostics identify
the landmark legend: positive axes use brighter and larger cubes; negative axes
use darker and smaller cubes. X is red, Y green, and Z blue.

## Claim Boundary

The fixture does not claim that this basis should become Tokimu's global
spatial convention. It deliberately keeps platform observation, interaction
policy, and camera basis separable so first-person, orbit, touch, gamepad, and
future chart-local views can choose different policies over shared or distinct
camera meaning.

CPU projection and picking evidence that consumes this exact pose and basis is
retained separately in
[`projection-picking-manifest.md`](projection-picking-manifest.md).
