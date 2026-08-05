"use strict";
(() => {
    const scriptUrl = new URL(document.currentScript?.src ?? window.location.href);
    const defaultFrameUrl = new URL("../assets/islands/ratatui-lab/index.html", scriptUrl);
    window.TokimuIslands.register("ratatui-lab", async ({ root, config: rawConfig, signal }) => {
        if (typeof WebAssembly === "undefined") {
            throw new DOMException("This browser does not provide WebAssembly.", "NotSupportedError");
        }
        const config = rawConfig;
        const mount = required(root, "[data-island-mount]");
        const fallback = required(root, ".island-fallback");
        const frame = document.createElement("iframe");
        const loaded = waitForFrame(frame, signal);
        prepareFrameLayout(mount, frame);
        frame.className = "ratatui-lab-frame";
        frame.title = "Tokimu Ratatui template laboratory";
        frame.src = new URL(config.frameUrl ?? defaultFrameUrl.href, scriptUrl).href;
        frame.tabIndex = 0;
        mount.replaceChildren(frame);
        mount.hidden = false;
        fallback.hidden = true;
        try {
            await loaded;
            frame.focus({ preventScroll: true });
            frame.contentWindow?.focus();
        }
        catch (error) {
            releaseFrame(frame, mount, fallback);
            throw error;
        }
        return { release: () => releaseFrame(frame, mount, fallback) };
    });
    function prepareFrameLayout(mount, frame) {
        // The generic island is a two-column fallback/status grid. Once activated,
        // the evidence frame owns the full row even if a stale site stylesheet is
        // still cached by the browser.
        mount.style.gridColumn = "1 / -1";
        mount.style.width = "100%";
        mount.style.minWidth = "0";
        frame.style.display = "block";
        frame.style.width = "100%";
        frame.style.maxWidth = "100%";
        frame.style.height = "clamp(38rem, 68vw, 52rem)";
        frame.style.border = "0";
    }
    function waitForFrame(frame, signal) {
        return new Promise((resolve, reject) => {
            const cleanup = () => {
                frame.removeEventListener("load", onLoad);
                frame.removeEventListener("error", onError);
                signal.removeEventListener("abort", onAbort);
            };
            const onLoad = () => { cleanup(); resolve(); };
            const onError = () => { cleanup(); reject(new Error("The Ratatui evidence frame failed to load.")); };
            const onAbort = () => { cleanup(); reject(new DOMException("Ratatui activation was cancelled.", "AbortError")); };
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
        if (!element)
            throw new Error(`Ratatui lab island requires ${selector}.`);
        return element;
    }
})();
