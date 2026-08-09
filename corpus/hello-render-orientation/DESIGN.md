# Hello Render Orientation

## Purpose

This native corpus consumer renders the shared AR-0021 orientation fixture
through Tokimu's WGPU backend. It does not define its own geometry or shader.

The window is a 4-by-3 matrix:

| Row | Transform case |
| --- | --- |
| 1 | Identity |
| 2 | Rotation and translation |
| 3 | X reflection without compensation |
| 4 | X reflection with one winding compensation |

| Column | Cull mode |
| --- | --- |
| 1 | None |
| 2 | Back |
| 3 | Front |

Green fragments report backend `front_facing`; magenta fragments report back
facing. Normal-derived brightness is separately retained. A visual capture is
evidence only when its backend identity and fixture layout are retained with it.

