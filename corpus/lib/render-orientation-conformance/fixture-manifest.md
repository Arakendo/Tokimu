# Directional Orientation Fixture Manifest

| Field | Retained value |
| --- | --- |
| Review pressure | AR-0021 and AR-0028 |
| Fixture kind | Asymmetric paired panels with caller-supplied UVs |
| Vertex topology | Unindexed triangle list; three triangles per panel |
| Source normal | `[0, 0, +1]` for every vertex |
| Left-panel winding | Counter-clockwise in source XY; geometric normal `+Z` |
| Right-panel winding | Clockwise in source XY; geometric normal `-Z` |
| Texture atlas | Generated RGBA8, 320 by 192, sRGB sampling |
| Front atlas region | `v = 0.0..0.5`, labeled `FRONT` |
| Back atlas region | `v = 0.5..1.0`, labeled `BACK` |
| Sampler | Caller material's default point/clamp sampler |
| Provider UV behavior | Supplied stream is consumed unchanged |

## Asymmetric Source Panels

Both panels use this five-point local shape and the triangle fan
`[0,1,2]`, `[0,2,3]`, `[0,3,4]`. The upper-right chamfer prevents a horizontal
reflection from looking like the original geometry.

| Vertex | Left position | Right position | UV |
| --- | --- | --- | --- |
| 0 | `[-0.95,-0.45,0]` | `[0.08,-0.45,0]` | `[0,1]` |
| 1 | `[-0.08,-0.45,0]` | `[0.95,-0.45,0]` | `[1,1]` |
| 2 | `[-0.08,0.22,0]` | `[0.95,0.22,0]` | `[1,0.26]` |
| 3 | `[-0.28,0.45,0]` | `[0.75,0.45,0]` | `[0.77,0]` |
| 4 | `[-0.95,0.45,0]` | `[0.08,0.45,0]` | `[0,0]` |

The right panel emits every triangle in reverse index order. The compensated
reflection case reverses every emitted triangle and its aligned UV entries
before applying the negative-X instance scale. No renderer code infers that
compensation from the transform determinant.

## Atlas Evidence

Each atlas half contains readable `FRONT` or `BACK`, `U- LEFT`, `RIGHT U+`,
`V- TOP UP`, `V+ BOTTOM`, and `N +Z` labels. The four UV corners are numbered
1–4 and independently colored:

| UV corner | RGBA8 |
| --- | --- |
| minimum U, minimum V | red `[255,55,45,255]` |
| maximum U, minimum V | yellow `[255,220,35,255]` |
| minimum U, maximum V | blue `[50,100,255,255]` |
| maximum U, maximum V | white `[245,245,245,255]` |

The corpus-local fragment shader selects the atlas half from WGPU's
`front_facing` fact. It does not change the supplied U coordinate and applies
only the explicit half-height mapping to V needed to select `FRONT` or `BACK`.
Thus geometric facing, U direction, V direction, and supplied-normal direction
remain independently visible.

## Transform And Cull Matrix

| Row | Source operation | Expected facing |
| --- | --- | --- |
| identity | identity instance | left front, right back |
| rotate-translate | translation `[0.08,-0.04]`, scale `[0.92,0.92]`, rotation `0.18` radians | left front, right back |
| reflect-x-uncompensated | scale `[-1,1]` | left back, right front; labels geometrically mirrored |
| reflect-x-compensated | source triangle reversal plus scale `[-1,1]` | left front, right back; texture reflection remains caller-visible |

Every row is submitted once under `CullMode::None`, `CullMode::Back`, and
`CullMode::Front`. Native and browser consumers obtain all meshes, pixels,
transforms, and viewport cells from `render-orientation-conformance`; neither
consumer owns a second fixture description.

## Claim Boundary

This manifest is deterministic structural evidence. Native and browser images
are retained visual observations of it and are not claimed pixel-identical.
The fixture does not admit a universal Tokimu world frame, automatic reflection
compensation, or a renderer-owned UV-orientation policy.
