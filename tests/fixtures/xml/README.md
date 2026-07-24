# XML Fixture Baseline

These fixtures establish the initial `xml-tools` profile before parser
integration. They are source fixtures, not browser DOM expectations.

| Case | Purpose | Expected initial result |
| --- | --- | --- |
| `well-formed/basic-elements.xml` | Elements, attributes, and an empty element | Accepted event sequence |
| `malformed/mismatched-close.xml` | Invalid closing element | Well-formedness diagnostic |
| `namespaces/prefixed-elements.xml` | Namespace declaration and prefixed element | Expanded-name event sequence |
| `references/character-references.xml` | Predefined and numeric character references | Decoded text event |
| `limits/deep-nesting.xml` | Configured nesting bound | Resource-limit diagnostic |
| `svg/commented-geometry.svg` | Comment containing SVG-looking geometry | Comment is not an SVG element |

The first parser adapter will convert this table into executable event and
diagnostic assertions. External entities, external DTD subsets, and resource
resolution are outside the initial profile.
