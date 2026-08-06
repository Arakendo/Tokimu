"use strict";

(() => {
  const scriptUrl = new URL(document.currentScript?.src ?? window.location.href);
  const defaultFrameUrl = new URL(
    "../assets/islands/runtime-observation/index.html",
    scriptUrl,
  );

  window.TokimuIslands.register("runtime-observation", async ({ root, config: rawConfig, signal }) => {
    if (typeof WebAssembly === "undefined") {
      throw new DOMException("This browser does not provide WebAssembly.", "NotSupportedError");
    }

    const config = rawConfig;
    const mount = required(root, "[data-island-mount]");
    const fallback = required(root, ".island-fallback");
    const frame = document.createElement("iframe");
    const releaseHeightSync = installFrameHeightSync(frame);
    const loaded = waitForFrame(frame, signal);

    prepareFrameLayout(mount, frame);
    frame.className = "runtime-observation-frame";
    frame.title = "Tokimu runtime observation workbench";
    frame.src = new URL(config.frameUrl ?? defaultFrameUrl.href, scriptUrl).href;
    frame.tabIndex = 0;
    mount.replaceChildren(frame);
    mount.hidden = false;
    fallback.hidden = true;

    try {
      await loaded;
      frame.focus({ preventScroll: true });
      frame.contentWindow?.focus();
    } catch (error) {
      releaseHeightSync();
      releaseFrame(frame, mount, fallback);
      throw error;
    }

    return {
      release: () => {
        releaseHeightSync();
        releaseFrame(frame, mount, fallback);
      },
    };
  });

  function prepareFrameLayout(mount, frame) {
    mount.style.gridColumn = "1 / -1";
    mount.style.width = "100%";
    mount.style.minWidth = "0";
    frame.style.display = "block";
    frame.style.width = "100%";
    frame.style.maxWidth = "100%";
    // The child reports its document height after WASM has rendered. This
    // fallback avoids a collapsed frame while the first report is in flight.
    frame.style.height = "48rem";
    frame.style.border = "0";
  }

  function installFrameHeightSync(frame) {
    const onMessage = (event) => {
      if (event.source !== frame.contentWindow) return;
      if (event.data?.type !== "tokimu-runtime-observation-height") return;

      const height = Number(event.data.height);
      if (!Number.isFinite(height) || height <= 0) return;
      frame.style.height = `${Math.ceil(height)}px`;
    };

    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }

  function waitForFrame(frame, signal) {
    return new Promise((resolve, reject) => {
      const cleanup = () => {
        frame.removeEventListener("load", onLoad);
        frame.removeEventListener("error", onError);
        signal.removeEventListener("abort", onAbort);
      };
      const onLoad = () => { cleanup(); resolve(); };
      const onError = () => { cleanup(); reject(new Error("The runtime observation workbench failed to load.")); };
      const onAbort = () => { cleanup(); reject(new DOMException("Runtime observation activation was cancelled.", "AbortError")); };
      frame.addEventListener("load", onLoad, { once: true });
      frame.addEventListener("error", onError, { once: true });
      signal.addEventListener("abort", onAbort, { once: true });
    });
  }

  function releaseFrame(frame, mount, fallback) {
    frame.src = "about:blank";
    frame.remove();
    mount.replaceChildren();
    mount.style.removeProperty("grid-column");
    mount.style.removeProperty("width");
    mount.style.removeProperty("min-width");
    mount.hidden = true;
    fallback.hidden = false;
  }

  function required(root, selector) {
    const element = root.querySelector(selector);
    if (!element) throw new Error(`Runtime observation island requires ${selector}.`);
    return element;
  }
})();
