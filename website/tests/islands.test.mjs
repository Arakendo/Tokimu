import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";

class FakeRoot {
  constructor() {
    this.dataset = {
      tokimuIsland: "asset-observation",
      state: "idle",
    };
    this.listeners = new Map();
    this.dispatched = [];
    this.stateLabel = { textContent: "" };
    this.detailLabel = { textContent: "" };
    this.activateButton = { disabled: false };
    this.resetButton = { hidden: true };
    this.config = {
      textContent: '{"schema":1,"activation":"explicit"}',
    };
  }

  querySelector(selector) {
    const elements = new Map([
      ["[data-island-status-state]", this.stateLabel],
      ["[data-island-status-detail]", this.detailLabel],
      ['[data-island-action="activate"]', this.activateButton],
      ['[data-island-action="reset"]', this.resetButton],
      ['script[type="application/json"][data-island-config]', this.config],
    ]);
    return elements.get(selector) || null;
  }

  addEventListener(type, listener) {
    this.listeners.set(type, listener);
  }

  removeEventListener(type, listener) {
    if (this.listeners.get(type) === listener) {
      this.listeners.delete(type);
    }
  }

  contains() {
    return true;
  }

  dispatchEvent(event) {
    this.dispatched.push(event);
  }

  click(action) {
    this.listeners.get("click")({
      target: {
        closest() {
          return { dataset: { islandAction: action } };
        },
      },
    });
  }
}

async function loadContract() {
  const source = await readFile(
    new URL("../docs/javascripts/islands.js", import.meta.url),
    "utf8",
  );
  const root = new FakeRoot();
  const windowListeners = new Map();
  const context = {
    AbortController,
    console,
    CustomEvent: class {
      constructor(type, options) {
        this.type = type;
        this.detail = options.detail;
      }
    },
    document: {
      querySelector() {
        return null;
      },
      querySelectorAll(selector) {
        return selector === "[data-tokimu-island]" ? [root] : [];
      },
    },
    window: {
      addEventListener(type, listener) {
        windowListeners.set(type, listener);
      },
    },
  };
  vm.runInNewContext(source, context);
  return { api: context.window.TokimuIslands, root, windowListeners };
}

test("an unavailable consumer reports unsupported and resets to idle", async () => {
  const { root } = await loadContract();

  root.click("activate");
  assert.equal(root.dataset.state, "unsupported");
  assert.match(root.detailLabel.textContent, /not published/);
  assert.equal(root.resetButton.hidden, false);

  root.click("reset");
  assert.equal(root.dataset.state, "idle");
  assert.equal(root.detailLabel.textContent, "No engine payload loaded.");
  assert.equal(root.resetButton.hidden, true);
});

test("a registered consumer mounts once and releases on reset", async () => {
  const { api, root } = await loadContract();
  let mounts = 0;
  let releases = 0;

  api.register("asset-observation", async ({ config, signal }) => {
    mounts += 1;
    assert.equal(config.schema, 1);
    assert.equal(signal.aborted, false);
    return {
      release() {
        releases += 1;
      },
    };
  });

  root.click("activate");
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(root.dataset.state, "ready");
  assert.equal(mounts, 1);

  root.click("reset");
  assert.equal(root.dataset.state, "idle");
  assert.equal(releases, 1);
});

test("a loader can report an unsupported browser explicitly", async () => {
  const { api, root } = await loadContract();

  api.register("asset-observation", async () => {
    const error = new Error("This browser does not provide WebAssembly.");
    error.name = "NotSupportedError";
    throw error;
  });

  root.click("activate");
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(root.dataset.state, "unsupported");
  assert.equal(
    root.detailLabel.textContent,
    "This browser does not provide WebAssembly.",
  );
});

test("page teardown releases listeners and marks islands unmounted", async () => {
  const { root, windowListeners } = await loadContract();

  windowListeners.get("pagehide")();
  assert.equal(root.dataset.state, "unmounted");
  assert.equal(root.listeners.has("click"), false);
});
