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

## Content Policy

The public site is curated. Repository plans, conversations, working notes,
and archives are not published automatically.

See `docs/Plans/tokimu-website.md` in the repository root for the implementation
plan and ownership boundaries.
