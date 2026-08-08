# Candidate API Ergonomics and Validation Ownership

## Scope

This compares the currently implemented B/C corpus-local API shapes. It is not
a proposal for a stable Tokimu math API and does not treat either candidate as
source-compatible with the full `glam` surface.

| Concern | B: private provider-backed vocabulary | C: original owned subset | Migration consequence |
| --- | --- | --- | --- |
| Value traits | Candidate values derive `Clone`, `Copy`, `Debug`, and `PartialEq` | Implemented values derive the same traits | Ordinary value passing and equality exist in both probes; debug formatting is not a stable contract. |
| Vector components | Private provider field; callers use accessors | `Vec3`/`Vec4` use owned scalar components | B turns direct component mutation into getter/reconstruction where callers need it; C preserves the scalar mutation shape. |
| Matrix final column | `w_axis()` / `set_w_axis(...)` | `w_axis()` / `set_w_axis(...)` over owned values | Both deliberately avoid copying the provider's public field contract. |
| Constants and types | Keeps five candidate names; `Vec2`/`Quat` are minimal compatibility probes | Implements only pressured `Vec3`, `Vec4`, and `Mat4` | Neither candidate claims replacement completeness from public names alone. |
| Operators | Delegates retained vector/matrix operators to pinned provider mechanics | Implements retained scalar/vector/matrix operators directly | Both satisfy the frozen caller/conformance subset, not an arbitrary trait family. |
| Validation and finite behavior | Delegates normalization and matrix behavior to pinned `glam` | Owns arithmetic and provisional singular-inverse behavior | B inherits provider changes; C must choose, test, diagnose, and maintain any eventual Tokimu contract. |
| Renderer boundary | Private unwrap preserves provider layout for the current facade | Explicit column reconstruction crosses into provider layout | B is mechanically simpler at this boundary; C makes representation ownership explicit. |

## Retained Evidence

- `tests/alternative_b_public_boundary.rs` compiles an external consumer of B
  without exposing a `glam` type or conversion helper.
- The FPS fixture records B's getter/reconstruction cost for direct component
  mutation; C retains direct component mutation.
- The hole-punch fixture records the explicit `w_axis` setter seam for B/C.
- The migration accounting record counts B/C renderer crossings and demonstrates
  finite matrix round trips at the retained boundary.
- Conformance labels provider observations separately from required finite
  caller behavior. C's all-NaN singular inverse remains provisional; neither
  candidate promotes it to a stable recovery contract.

## Conclusion

B is a plausible transitional vocabulary layer, but it retains provider
mechanics and wrapper ergonomics. C offers implementation ownership and more
direct scalar access, while accepting numerical, performance, and maintenance
responsibility. These are tradeoffs to preserve through the post-DOOM revisit,
not grounds to promote either candidate today.
