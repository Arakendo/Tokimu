# Alternative D Is Blocked For Hello 3D Mono

The rotating-cube corpus path requires `Mat4` rotation constructors and point /
vector transforms. Alternative D intentionally retains only its bounded `Vec3`
slice, so this case is blocked rather than replaced with provider math.

Unblocking this case requires a reviewed, evidence-backed decision to expand D
with exact matrix-source provenance and the associated maintenance obligation.
