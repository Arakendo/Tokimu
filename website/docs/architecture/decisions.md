---
title: How Tokimu Makes Architectural Decisions
description: Tokimu uses corpus pressure, implementation evidence, Architectural Reviews, and ADRs to admit durable meaning.
---

<p class="page-kicker">Architecture / method</p>

# How Tokimu makes architectural decisions

Tokimu treats architecture as something implementation must interrogate. A
useful idea does not become foundational merely because it is convenient or
common in other engines.

## The evidence lifecycle

<div class="decision-flow" aria-label="Tokimu architectural decision lifecycle">
  <div><span>01</span><strong>Need appears</strong><p>A concrete application or corpus exposes pressure.</p></div>
  <div><span>02</span><strong>Focused proof</strong><p>The smallest honest example tests one boundary.</p></div>
  <div><span>03</span><strong>Evidence accumulates</strong><p>Independent producers or consumers reveal repeated semantics.</p></div>
  <div><span>04</span><strong>Review</strong><p>An Architectural Review records findings, uncertainty, and deferral.</p></div>
  <div><span>05</span><strong>Admission</strong><p>Durable ownership is accepted only when the evidence warrants it.</p></div>
  <div><span>06</span><strong>ADR</strong><p>A binding decision records the boundary and its consequences.</p></div>
</div>

## Corpus pressure

A focused corpus entry is an executable architectural sentence. It asks whether
one behavior can be expressed naturally through the intended ownership
boundary.

External data corpora ask whether implementations survive inputs Tokimu did not
design for itself. Consumer corpora ask whether several contracts compose into
an ordinary downstream application without privileged access.

## Reviews are allowed to say “not yet”

Architectural Reviews preserve evidence without forcing a permanent answer.
They can accept a direction while deferring crate extraction, reopen when new
corpus pressure appears, or reject an attractive abstraction whose ownership
remains unclear.

## ADRs are binding

An ADR records an accepted architectural decision. Local implementation should
not quietly work around it. If later evidence changes the decision, Tokimu
updates or supersedes the ADR while retaining the earlier history.

## Why the website exposes limitations

Tokimu distinguishes observations from guarantees. Public capability labels
such as **Renderable**, **Previewable**, **Inspected**, **Experimental**, and
**Deferred** communicate what the current evidence actually proves.

The goal is not to make every capability look complete. The goal is to make
every claim inspectable.
