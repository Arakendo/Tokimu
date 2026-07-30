(() => {

type AsteroidsIslandConfig = {
  frameUrl?: string;
};

const scriptUrl = new URL(
  (document.currentScript as HTMLScriptElement | null)?.src ?? window.location.href,
);
const defaultFrameUrl = new URL(
  "../assets/islands/asteroids-game/index.html",
  scriptUrl,
);

window.TokimuIslands.register(
  "asteroids-game",
  async ({ root, config: rawConfig, signal }) => {
    if (typeof WebAssembly === "undefined") {
      throw new DOMException(
        "This browser does not provide WebAssembly.",
        "NotSupportedError",
      );
    }

    const config = rawConfig as AsteroidsIslandConfig;
    const mount = required<HTMLElement>(root, "[data-island-mount]");
    const fallback = required<HTMLElement>(root, ".island-fallback");
    const frame = document.createElement("iframe");
    const loaded = waitForFrame(frame, signal);

    frame.className = "asteroids-frame";
    frame.title = "Playable Tokimu Asteroid field";
    frame.src = new URL(config.frameUrl ?? defaultFrameUrl.href, scriptUrl).href;
    frame.tabIndex = 0;
    frame.setAttribute("allow", "fullscreen");

    mount.replaceChildren(frame);
    mount.hidden = false;
    fallback.hidden = true;

    try {
      await loaded;
      frame.focus({ preventScroll: true });
      frame.contentWindow?.focus();
    } catch (error) {
      releaseFrame(frame, mount, fallback);
      throw error;
    }

    return {
      release() {
        releaseFrame(frame, mount, fallback);
      },
    };
  },
);

function waitForFrame(frame: HTMLIFrameElement, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      frame.removeEventListener("load", onLoad);
      frame.removeEventListener("error", onError);
      signal.removeEventListener("abort", onAbort);
    };
    const onLoad = () => {
      cleanup();
      resolve();
    };
    const onError = () => {
      cleanup();
      reject(new Error("The Asteroids evidence frame failed to load."));
    };
    const onAbort = () => {
      cleanup();
      reject(new DOMException("Asteroids activation was cancelled.", "AbortError"));
    };

    frame.addEventListener("load", onLoad, { once: true });
    frame.addEventListener("error", onError, { once: true });
    signal.addEventListener("abort", onAbort, { once: true });
  });
}

function releaseFrame(
  frame: HTMLIFrameElement,
  mount: HTMLElement,
  fallback: HTMLElement,
): void {
  frame.src = "about:blank";
  frame.remove();
  mount.replaceChildren();
  mount.hidden = true;
  fallback.hidden = false;
}

function required<T extends Element>(root: ParentNode, selector: string): T {
  const element = root.querySelector<T>(selector);
  if (!element) {
    throw new Error(`Asteroids island requires ${selector}.`);
  }
  return element;
}

})();
