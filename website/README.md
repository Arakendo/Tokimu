# Tokimu Website

This directory contains the curated public documentation site for
`tokimuengine.org`.

MkDocs owns documents, routing, metadata, and static fallback content. Tokimu
WASM may later mount into explicitly declared evidence regions, but the site
must remain useful without JavaScript, WebAssembly, WebGPU, or a live renderer.

## Local Preview

Create a local environment and install the bounded site dependency:

```powershell
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r requirements.txt
.\.venv\Scripts\python.exe -m mkdocs serve -f mkdocs.yml
```

Build the static output:

```powershell
.\.venv\Scripts\python.exe -m mkdocs build -f mkdocs.yml --strict
```

Build the optional Tokimu-powered evidence island:

```powershell
npm install
pwsh -NoProfile -File .\scripts\build-interactive.ps1
```

The interactive build compiles the TypeScript browser adapter, reuses the
ASP.NET asset-workbench Rust/WASM engine, and refreshes the committed generated
assets under `docs/assets/islands/asset-observation`. Ordinary MkDocs builds do
not require Rust, Cargo, or `wasm-bindgen`.

Generated output is written to `target/website` and is not committed.

The generated root includes:

- `.nojekyll`, emitted by the configured MkDocs hook to keep GitHub Pages from
  applying Jekyll processing; and
- `CNAME`, which declares `tokimuengine.org` as the canonical Pages domain.

## Deployment

Pushes to `main` that change `website/` run
`.github/workflows/deploy-website.yml`. The workflow builds the static site and
deploys `target/website` as a GitHub Pages artifact.

There is no generated website branch. Repository source remains on `main`, and
GitHub Pages deployment history provides the rollback boundary.

The repository's Pages source must be configured as **GitHub Actions** in the
GitHub repository settings.

### Deployment Ownership

Responsibility is deliberately split:

- repository source, MkDocs configuration, generated-asset contracts, and the
  Pages workflow are owned in this repository;
- GitHub Pages owns production artifact hosting and the `.org` certificate;
- the domain provider owns `.com` and `.net` forwarding; and
- the generated website never becomes an architectural source of truth.

`tokimuengine.org` is canonical. Forwarding for `tokimuengine.com` and
`tokimuengine.net` is configured externally and must be verified after DNS and
edge propagation before path-preserving HTTPS redirects are treated as
complete.

### Rollback

Production is restored through ordinary repository history, not by editing the
generated Pages artifact:

1. Identify the last known-good website deployment and its source commit.
2. Revert the offending source commit or apply a corrective commit on `main`.
3. Run the strict local build and website tests.
4. Push the restoration commit and allow the normal Pages workflow to publish
   a new immutable deployment artifact.
5. Verify the `.org` canonical URL, static fallback, and affected route.

This keeps rollback reviewable and preserves the relationship between public
claims and their source revision. A historical artifact may be inspected during
diagnosis, but source is restored through Git rather than by making an
unrecorded production-only change.

## Content Policy

The public site is curated. Repository plans, conversations, working notes,
and archives are not published automatically.

See `docs/Plans/tokimu-website.md` in the repository root for the implementation
plan and ownership boundaries.
