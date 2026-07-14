# Other Ideas

This is a side note, not a commitment in the current roadmap.

The live native proof window is intentionally blank; its title carries the
state readout so the empty surface reads as deliberate instead of unfinished.

## .NET/C# bindings are plausible

Yes. In fact, .NET/C# looks like one of the more realistic ecosystems for Tokimu to support later.

There are a few ways to do it, each with different tradeoffs.

### Option 1: C ABI

This is the simplest and most durable bridge.

```text
Tokimu Core (Rust)
        ▲
        │ C ABI
        ▼
C#
Python
C++
Swift
Go
```

Rust exports a stable C API, and C# uses `DllImport` or `LibraryImport` to call into a native Rust library.

Conceptually, that API could look like this:

```text
TokimuEngine* tokimu_create(...);

void tokimu_update(
    TokimuEngine* engine,
    double dt);

void tokimu_destroy(
    TokimuEngine* engine);
```

C# then wraps that in a more idiomatic object-oriented API.

This keeps the Rust core completely independent of .NET.

### Option 2: WebAssembly

Tokimu is already thinking about WebAssembly, so this is another plausible path.

```text
Rust
↓
WASM
↓
Blazor
```

A Blazor application could host the Tokimu runtime in the browser.

That is especially interesting for:

- editors
- visualization tools
- simulation dashboards
- digital twins

### Option 3: Native .NET bindings

A later step could be a dedicated package such as `Tokimu.NET`:

```text
Tokimu.Core.dll

↓

Tokimu.NET
```

The goal would be an idiomatic C# surface, not a raw C handle API.

A C# developer might write something like this:

```text
using var engine = new Engine();

engine.Update(deltaTime);

engine.World.Create<Entity>();
```

The wrapper would quietly forward those calls into Rust.

## Why this matters

This becomes more interesting when you imagine a stack like this:

```text
WPF

↓

Tokimu.NET

↓

Tokimu Engine Kernel (Rust)

↓

wgpu
```

Or:

```text
WinForms

↓

Tokimu.NET

↓

Tokimu Engine Kernel
```

Or:

```text
Avalonia

↓

Tokimu.NET

↓

Tokimu Engine Kernel
```

The UI toolkit stays separate from the simulation kernel.

That matches the broader goal of letting people build applications, not just engine internals.

A lot of engineering software is written in C#:

- CAD utilities
- industrial HMI software
- PLC tools
- GIS viewers
- internal engineering applications
- simulation frontends

Giving those developers a way to use Tokimu without writing Rust would broaden its reach.

## Architectural implication

If bindings are in Tokimu's future, it reinforces a principle that already matters:

**Own the API, do not expose Rust internals.**

Instead of exposing Rust-specific types like this:

```text
EntityId
ResourceHandle<T>
```

prefer a language-neutral API model that every binding can present idiomatically:

```text
Engine

World

Entity

Component

Resource

System

Diagnostics
```

Then each wrapper language can adapt those concepts to its own style.

## The larger symmetry

This is the bigger picture:

```text
                 C#
                  │
TypeScript ───────┤
                  │
Python ───────────┤
                  │
                  ▼
         Tokimu Engine Kernel
                  │
          Rendering / World / Runtime
                  │
             Platform Adapters
```

Tokimu would not be tied to one programming language.

That is usually a good sign for mature infrastructure: multiple languages can consume the same core runtime.

I would not build C# bindings soon. Tokimu still has engine work to finish first. But keeping .NET in mind while designing public APIs is a useful pressure test. If the core stays clean enough that wrapping it for C# feels straightforward, that usually means the architecture is staying well-factored instead of accidentally becoming Rust-specific.

## Tokimu.Quake is a plausible companion idea

Quake 1 is interesting because it uses vertex animation rather than skeletal animation.

Each animation frame is literally another set of vertex positions, and runtime motion comes from interpolating between frames.

```text
Frame 0
    Vertices

↓

Frame 1
    Vertices

↓

Frame 2
    Vertices
```

That makes it a very small and self-contained first animation system.

Instead of starting with a full skeletal stack:

```text
Skeleton
↓
Bone hierarchy
↓
Animation clips
↓
Blend trees
↓
IK
↓
Retargeting
```

you can prove something much simpler:

```text
AnimationClip
↓
Frame 12
↓
Frame 13
↓
Interpolation
↓
Mesh
```

That keeps the problem small while still forcing Tokimu to treat animation as a first-class concept.

The useful architectural lesson is not that vertex animation should be the whole model forever. The useful lesson is that Tokimu may want a more abstract evaluator-style concept that advances time and produces a new evaluated state.

That could eventually look more like this:

```text
Evaluator
↓
State
```

or, if the domain wants a more explicit name:

```text
AnimationClip
↓
AnimationState
↓
Pose
```

The important part is that "pose" does not have to mean the same thing in every domain.

For Quake-style animation:

```text
Pose
↓
Vertex positions
```

For skeletal animation:

```text
Pose
↓
Bone transforms
```

For another evaluated system, it could be something else entirely.

That is the same kernel idea in a different form: the core defines the concept of time-based evaluation, and each domain supplies its own evaluated state.

There is also an asset-ownership lesson here. If Tokimu ever supports animation more directly, it likely wants a separation like this:

```text
Mesh
↓
Animation
↓
Animator
↓
Renderable
```

That separation is valuable even if the implementation stays simple at first.

If Tokimu later adds skeletal animation, morph targets, or procedural animation, those can be additional backends or evaluated-state paths rather than a rewrite of the whole model.

That makes Quake less of a final animation design and more of a useful proof target. It can teach Tokimu what the right animation abstractions should look like without locking the engine into Quake's limitations.

## What a first pass could look like

If Tokimu.Quake ever becomes a real companion project, the smallest useful version should probably stay close to the current engine shape.

The first goal would not be a general animation editor. It would be a loader and evaluator for one specific asset style:

```text
Animation asset
↓
Frame list
↓
Interpolated pose
↓
Renderable mesh
```

That keeps the boundary clear.

Tokimu would still own the timing, scheduling, and render submission. The Quake-specific layer would only answer one question: given time, what evaluated vertex state should this mesh present right now?

That suggests a small split like this:

```text
Asset
↓
AnimationClip
↓
AnimationState
↓
EvaluatedMesh
```

In practice, that could mean:

- an asset format that stores keyed vertex frames
- a loader that turns those frames into an engine-owned clip structure
- a small evaluator that advances clip time and picks or interpolates frames
- a renderable that consumes the evaluated mesh state without knowing how it was produced

The key is to avoid hard-coding Quake into the renderer. The renderer should still just see a mesh and draw commands. The animation system should own the state transition.

That also means the project can stay honest about scope. Tokimu.Quake would not need to solve skeletal animation, animation graphs, or retargeting on day one. It would only need to prove that Tokimu can own a time-based mesh-evaluation path cleanly.

If that works, later animation styles become additions rather than replacements.
