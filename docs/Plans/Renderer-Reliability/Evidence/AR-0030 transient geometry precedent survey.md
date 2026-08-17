# AR-0030 Transient Geometry Precedent Survey

## Question

What can established rendering systems teach the AR-0030 G1--G4 comparison
without allowing another engine's API shape to decide Tokimu's contract?

The immediate corpus pressure is deliberately small: two Doom-owned,
camera-dependent sky-depth declarations contain 12 vertices and four triangles,
refer to one durable sky material, and require no durable mesh identity. The
question is about lifetime and ownership, not about importing Doom semantics.

## Findings

### bgfx: bounded transient allocation is established practice

bgfx exposes transient vertex and index buffers as per-frame temporary
allocations. Capacity can be queried, allocation is bounded, and the storage is
recycled after the frame boundary.

This is useful evidence for G3's provider-internal mechanism. It is not evidence
that Tokimu should define prepared presentation identity as `frame-local`:
multiple views, retries, offscreen work, and retained observations may not share
one semantic lifetime merely because a backend services them during one frame.

### wgpu: staging reuse and semantic geometry identity are separate

wgpu's `StagingBelt` and queue write helpers demonstrate reusable staging
allocations with explicit finish, submission, and recall behavior. They solve
copy/upload economics and reuse; they do not assign domain or presentation
identity to the bytes being copied.

This supports keeping G2 submission-local identity above provider staging. A
WGPU backend may realize a G2 payload through an arena or staging belt without
making that implementation mechanism part of Tokimu's meaning.

### Vulkan: reuse is governed by completion, not naming convenience

Vulkan's synchronization and swapchain guidance makes the underlying safety
rule explicit: command and resource storage cannot be overwritten while the
GPU may still consume it. Reuse must follow the relevant in-flight completion,
not a naive CPU frame counter.

This is evidence against a public contract whose only guarantee is “valid for
this frame.” A provider may need fences, epochs, recalls, or multiple backing
allocations even when the caller sees one immutable bounded submission.

### Bevy: extraction/preparation can remain outside final rendering

Bevy separates read-only extraction from a main world, render preparation,
render execution, and cleanup. This supports the broader AR-0030 direction:
domain/application state can produce presentation work without the renderer
owning simulation truth.

A separate render world or render graph is not earned by the current Tokimu
evidence. Render graphs primarily organize passes and dependencies; they do not
by themselves answer whether four view-local triangles are durable resources.

## Tokimu Interpretation

The precedents converge on a separation rather than a borrowed API:

```text
domain/source correlation
    durable for evidence and diagnostics

submission-local presentation identity
    bounded by one immutable handoff/version

persistent renderer resource identity
    materials, textures, and durable meshes across submissions

provider staging identity
    buffers/epochs/fences used to realize the handoff safely
```

The leading private hypothesis therefore remains G2:

- the submission owns bounded geometry payloads;
- ordered draws reference identities local to that submission;
- durable material identity remains separate;
- local geometry identities cannot resolve in another submission;
- providers may implement the payload using G1-style inline writes or G3-style
  transient pools;
- G4 persistent replacement remains a negative/control path.

The evidence does **not** admit `TransientMesh`, a render graph, a render world,
or a universal frame arena. It only justifies making the G2 lifetime model
executable inside the corpus before testing GPU realization.

## Sources

- [bgfx internals: transient buffers](https://bkaradzic.github.io/bgfx/internals.html)
- [bgfx API: transient allocation and capacity](https://bkaradzic.github.io/bgfx/bgfx.html?highlight=limits)
- [wgpu `StagingBelt`](https://docs.rs/wgpu/latest/wgpu/util/struct.StagingBelt.html)
- [wgpu `Queue`](https://docs.rs/wgpu/latest/wgpu/struct.Queue.html)
- [Vulkan rendering and presentation](https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/02_Rendering_and_presentation.html)
- [Vulkan swapchain semaphore reuse](https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html)
- [Vulkan synchronization guide](https://docs.vulkan.org/guide/latest/synchronization.html)
- [Bevy `Extract`](https://docs.rs/bevy/latest/bevy/render/struct.Extract.html)
- [Bevy render systems](https://docs.rs/bevy/latest/bevy/render/enum.RenderSystems.html)
- [Bevy render graph](https://docs.rs/bevy/latest/bevy/render/prelude/struct.RenderGraph.html)

## Disposition

Use precedent to constrain the corpus experiment, not to select Tokimu's public
vocabulary. Implement and test G2 privately; treat G3 as a likely backend
realization whose safe reuse is provider-owned and completion-aware.
