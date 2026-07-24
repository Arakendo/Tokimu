# XML Profile Feature Matrix

This matrix describes the initial `xml-tools` profile, not the complete W3C
suite.

| Capability | Status | Selection evidence |
| --- | --- | --- |
| UTF-8 source bytes | supported | `eduni-errata-2e-e57` |
| Elements, attributes, and empty elements | supported | `eduni-errata-2e-e57` |
| Namespace-expanded identities | supported | `eduni-errata-2e-e57` |
| Matching end tags | rejected when malformed | `xmltest-not-wf-sa-039` |
| UTF-16 source bytes | explicitly unsupported | `eduni-errata-2e-e61` |
| Invalid namespace QName | deferred diagnostic mapping | `eduni-namespaces-1-0-013` |
| DOCTYPE and DTD processing | explicitly unsupported | intentionally excluded from v1 accepted cases |
| External entities and external resource resolution | explicitly unsupported | intentionally excluded from v1 accepted cases |

`UnsupportedByProfile` means the result is deliberate and stable for this
profile. It must not be counted as an implementation regression.
