# Hello Hole Punch

## Purpose

`hello-hole-punch` is a focused GLB scene-loading corpus example for the
user-supplied `corpus/assets/GLB/hole_punch1.glb` asset.

It proves that the current GLB corpus decoder can:

- inspect and linearly sample source-level translation animation metadata;
- decode every triangle primitive in the asset;
- apply declared scene-node transforms; and
- submit the resulting multi-part animated scene to the existing 3D renderer.

## Animation Finding

The inspected `hole_punch1.glb` document has five named, linearly interpolated
translation clips: `step1` through `step5`. The example cycles these clips in
source order, retains each completed clip's final translation while the next
clip runs, and reports the active clip plus held-step count in the window
title. A presentation-only X-axis rotation places the tool on its back before
the bind pose is normalized; it does not alter source node or animation
semantics. Camera orbit is separate inspection motion.

## Non-Goals

- glTF rotation, scale, weights, spline interpolation, or skinning;
- material or texture import;
- promoting GLB import into a native Tokimu capability;
- replacing `hello-glb`.
