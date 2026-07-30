---
title: Interactive Island Contract
description: How bounded Tokimu WASM consumers attach to the static website.
---

# Interactive island contract

Tokimu-powered regions are optional guests inside an ordinary static page.
The page owns durable knowledge; an island may add executable evidence after
explicit activation.

## Declarative boundary

An island root declares a semantic consumer name and an initial lifecycle
state:

```html
<section
  data-tokimu-island="asset-observation"
  data-state="idle"
>
  <!-- Static fallback content remains readable here. -->
</section>
```

Configuration is inert JSON owned by the page:

```html
<script type="application/json" data-island-config>
  {
    "schema": 1,
    "fixture": "asset-observation-v1",
    "activation": "explicit",
    "maxBytes": 8388608
  }
</script>
```

The browser adapter passes this data to the registered consumer. It does not
interpret importer, simulation, or presentation meaning.

## Lifecycle

Each island has an independent controller and one of these visible states:

| State | Meaning |
| --- | --- |
| `idle` | Static content is active and no interactive resources are owned. |
| `loading` | The registered consumer is starting under an abort signal. |
| `ready` | The consumer mounted and owns a bounded release callback. |
| `unsupported` | No compatible consumer is published for this build or browser. |
| `failed` | Startup failed and the static explanation remains available. |
| `unmounted` | Event, WASM, and renderer resources have been released. |

Reset passes through `unmounted` before returning to `idle`. Navigation away
from the page disposes every discovered island.

## Loader boundary

An interactive bundle registers itself by semantic island name:

```javascript
window.TokimuIslands.register("asset-observation", async (context) => {
  const consumer = await mountTokimuConsumer(context);
  return {
    release() {
      consumer.dispose();
    },
  };
});
```

The context contains only the island root, parsed configuration, and an
`AbortSignal`. Independent islands do not share application state.

## Failure contract

Missing or failed interactive code never removes the static fallback. State
changes are exposed through textual status and the bubbling
`tokimu:island-state` event. A consumer must report unsupported behavior rather
than silently substituting browser-native import or rendering semantics.

This contract is scaffolded and exercised by the homepage. A real Tokimu WASM
consumer remains pending.
