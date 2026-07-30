---
title: Tokimu
description: Tokimu is a Rust-native runtime for interactive engines, simulations, and applications.
---

<section class="hero">
  <div class="hero-copy">
    <p class="eyebrow"><span>Runtime</span><span>World</span><span>Evidence</span></p>
    <h1>Build interactive systems around <em>meaning</em>, not machinery.</h1>
    <p class="hero-lede">
      Tokimu is a Rust-native runtime for interactive engines, simulations, and
      applications. It keeps simulation truth separate from presentation,
      platforms, tools, and providers.
    </p>
    <div class="hero-actions">
      <a class="button button-primary" href="getting-started/">Explore Tokimu</a>
      <a class="button button-secondary" href="architecture/">Read the architecture</a>
    </div>
  </div>

  <div class="hero-instrument" aria-label="Tokimu architecture summary">
    <div class="instrument-header">
      <span>Observation 001</span>
      <strong>World state / stable</strong>
    </div>
    <div class="instrument-body">
      <div class="orbit orbit-outer"></div>
      <div class="orbit orbit-middle"></div>
      <div class="orbit orbit-inner"></div>
      <div class="instrument-core">
        <span>Truth</span>
        <strong>World</strong>
      </div>
      <span class="instrument-label label-runtime">Runtime</span>
      <span class="instrument-label label-present">Presentation</span>
      <span class="instrument-label label-observe">Diagnostics</span>
    </div>
    <div class="instrument-footer">
      <span>Rust native</span>
      <span>WASM planned</span>
      <span>Provider neutral</span>
    </div>
  </div>
</section>

<section class="proof-strip" aria-label="Project principles">
  <div><span>01</span><strong>World-first</strong><p>Simulation owns truth.</p></div>
  <div><span>02</span><strong>Provider-neutral</strong><p>Mechanisms stay below meaning.</p></div>
  <div><span>03</span><strong>Evidence-driven</strong><p>Corpus pressure shapes APIs.</p></div>
  <div><span>04</span><strong>Explicit limits</strong><p>Deferred means deferred.</p></div>
</section>

## One runtime, many expressions

Tokimu is designed around a reusable engine kernel rather than one application
genre. Games are important, but the same runtime concepts should also support
technical simulators, creative tools, industrial dashboards, and other
interactive systems.

<div class="card-grid">
  <article class="feature-card">
    <span class="card-index">A</span>
    <h3>Simulation</h3>
    <p>Entities, relationships, rules, time, and commands form an inspectable source of truth.</p>
  </article>
  <article class="feature-card">
    <span class="card-index">B</span>
    <h3>Presentation</h3>
    <p>Text, vectors, meshes, and future XR views consume meaning without owning it.</p>
  </article>
  <article class="feature-card">
    <span class="card-index">C</span>
    <h3>Observation</h3>
    <p>Diagnostics expose measured behavior, unsupported cases, and sustained performance pressure.</p>
  </article>
</div>

## Evidence over claims

Tokimu's corpus is executable architectural evidence. Focused entries prove
individual boundaries. Data corpora pressure importers with external files.
Consumer corpora test whether real application shapes can compose the public
contracts cleanly.

<div class="evidence-panel">
  <div>
    <p class="eyebrow">Current evidence vocabulary</p>
    <h3>Not everything is simply “supported.”</h3>
  </div>
  <dl class="state-list">
    <div><dt>Renderable</dt><dd>Admitted semantics can be lowered and presented.</dd></div>
    <div><dt>Previewable</dt><dd>A bounded diagnostic preview exists; semantics remain deferred.</dd></div>
    <div><dt>Inspected</dt><dd>Structure can be decoded and reported without a rendering claim.</dd></div>
    <div><dt>Experimental</dt><dd>The behavior runs through an incubating boundary.</dd></div>
  </dl>
</div>

## The first public instrument

<section class="island-stage" data-tokimu-island="asset-observation" data-state="idle">
  <div class="island-fallback">
    <p class="eyebrow">Interactive evidence / scaffolded</p>
    <h3>Asset observation workbench</h3>
    <p>
      This region will mount a bounded Tokimu WASM consumer. Until that consumer
      is integrated, the static explanation remains the authoritative content.
    </p>
    <button class="button button-disabled" type="button" disabled>
      WASM consumer pending
    </button>
  </div>
  <div class="island-status" role="status" aria-live="polite">
    <span>Idle</span>
    <span>No engine payload loaded</span>
  </div>
</section>

<div class="next-step">
  <span>Next</span>
  <div>
    <strong>See how architectural decisions are earned.</strong>
    <p>Tokimu uses corpus pressure, review records, and ADRs to separate observation from commitment.</p>
  </div>
  <a href="architecture/decisions/" aria-label="Read how Tokimu makes architectural decisions">Read the method →</a>
</div>
