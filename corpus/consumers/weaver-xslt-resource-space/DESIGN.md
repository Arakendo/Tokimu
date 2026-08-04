# Weaver XSLT Resource Space Consumer

## Primary Claim

Can a TypeScript XSLT consumer execute selected XML and stylesheet bytes while
keeping XSLT semantics in Weaver and resource identity in Tokimu?

## Current Proof

The initial fixture is a source-buffer proof only:

```text
selected source.xml + stylesheet.xsl
        |
        v
Weaver XsltProcessor
        |
        v
expected.xml comparison
```

`related.xml` is intentionally retained but not resolved yet. A real
same-folder lookup must wait for Weaver's public resolver contract; direct
filesystem reads would prove Node access, not the Resource Space boundary.

## Ownership

- Weaver owns XSLT/XPath interpretation and execution.
- The TypeScript consumer owns its selected fixture session and result
  comparison.
- Tokimu Resource Space owns eventual qualified identity and retained bytes.
- This consumer does not make any Tokimu crate depend on Node or npm.

## Non-Goals

- No general URI resolver.
- No filesystem or network authorization policy.
- No XSLT provider admission.
- No Resource Space API change.
