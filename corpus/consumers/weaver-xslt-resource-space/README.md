# Run The Baseline

Install Weaver's pinned development dependencies once:

```powershell
npm --prefix third-party/weaver-xslt ci
```

Then run the consumer from the repository root:

```powershell
npm --prefix third-party/weaver-xslt exec -- tsx "$PWD/corpus/consumers/weaver-xslt-resource-space/run.ts"
```

The runner writes a non-versioned observation to
`target/weaver-xslt-resource-space/baseline.json`.

This baseline verifies source-buffer composition only. It does not claim
Resource Space URI-resolution integration until Weaver exposes the documented
public `ResourceResolver` seam.
