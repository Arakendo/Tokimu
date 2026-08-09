# Alpha-Policy Fixture And Scene Manifest

## Fixture Contract

All fixtures are first-party exact RGBA8 arrays produced by
`hello_alpha_policy::fixtures()`. There is no encoded-image decoder in this
baseline, so alpha values cannot change through PNG color-type, palette, or
transparency-chunk behavior. Fingerprints are BLAKE3 over width and height as
little-endian `u32` values followed by the exact row-major RGBA8 payload.

| Fixture | Dimensions | Alpha distribution | BLAKE3 | Purpose |
| --- | ---: | --- | --- | --- |
| `opaque-control` | 4x1 | `255:4` | `3678aa1245e5e36ceb1f5b59dbc60e6da129db027ef6df452f600c644e99d129` | unchanged opaque baseline |
| `binary-mask` | 4x1 | `0:2, 255:2` | `37dc5c494f2394b7c7c99eca6cc800f039975fb6add1e48868fbc965657fa48e` | categorical keep/discard |
| `threshold-boundary` | 5x1 | `0:1, 127:1, 128:1, 129:1, 255:1` | `62558ade2bf5d4ca32c79d234ce4f282f37adf01b551a52907fb60c26df69e2d` | below/equal/above `128/255` |
| `continuous-gradient` | 256x1 | every value `0..=255` exactly once | `7e57ab1608b24e89af1dda5c1ff51cfe9f8e74fe9d063a42cd1b371debbff6bd` | every alpha byte 0 through 255 |
| `mixed-alpha` | 5x1 | `0:1, 64:1, 128:1, 192:1, 255:1` | `2d82b95538bf2af33e88a9eb1bd1a2de73e9a1a15d3305f1268c048e7c9fc4dd` | identical bytes under all profiles |
| `colored-transparent` | 4x1 | `0:4` | `cc4f041a142a01cff05acf6c8967921cde5c7a56ebb367f656e6ec8d190ca572` | nonzero RGB with zero alpha |

These values were generated after `cargo test -p hello-alpha-policy` passed all
fixture and oracle tests. The executable report remains the reproducible source
for the complete 256-value gradient distribution.

## Frozen Scene Matrix

| ID | Fixture | Ordered draws | BLAKE3 | Variable under test |
| --- | --- | --- | --- | --- |
| `same-texture-three-profiles` | `mixed-alpha` | `opaque`, `cutout`, `blend` | `1ca0df91e92939a737f72364b785069edd14165e5a8d47963067840c7ea95da2` | profile only |
| `cutout-over-opaque` | `binary-mask` | `background`, `cutout` | `86fc9dc54299fa0a1c78c6d4646326dd88d07336fdb48a6d2cae86345ab4b794` | categorical foreground |
| `blend-over-opaque` | `continuous-gradient` | `background`, `blend` | `41343caddde50643d69e5e8f83273f83159cec87377ea21411947fc69659ac83` | continuous foreground and depth write |
| `overlapping-blend-back-to-front` | `mixed-alpha` | `background`, `far-blend`, `near-blend` | `3e8dd5abc1a3d0b97f55cfbd557a31d74ea96b9a1139c28ec4ade41775e72a5f` | caller order |
| `overlapping-blend-front-to-back` | `mixed-alpha` | `background`, `near-blend`, `far-blend` | `a49604bdb4d6b053174d9ac420f06bdf6fd8cd05bfcdeb655e449a5f060ea6d3` | reversed caller order |
| `cutout-blend-intersection` | `mixed-alpha` | `background`, `blend`, `cutout` | `f129d02267efa29405a5bed436fcdac306e640baafd3eeeef2eb6f35d69fd196` | capability interaction |
| `identical-depth-overlap` | `mixed-alpha` | `background`, `first`, `second` | `4ab5d287b04098be61acb1b27af6fd392ec3b00729e80dd9971cfd93bf0992fc` | depth comparison, not alpha inference |

Draw order is part of each source observation. Later native/browser consumers
must preserve these identities and record any deliberate depth-write variant.
No visual image is accepted as evidence that an unrecorded order was correct.
The two overlapping-blend cases retain identical per-draw transforms, scale,
and depth; only the `near-blend`/`far-blend` submission sequence changes.
