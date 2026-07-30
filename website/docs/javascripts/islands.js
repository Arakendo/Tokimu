(() => {
  const ISLAND_STATES = new Set([
    "idle",
    "loading",
    "ready",
    "unsupported",
    "failed",
    "unmounted",
  ]);
  const loaders = new Map();
  const controllers = new Map();

  const navToggle = document.querySelector(".nav-toggle");
  const navigation = document.querySelector(".site-navigation");

  if (navToggle && navigation) {
    navToggle.addEventListener("click", () => {
      const expanded = navToggle.getAttribute("aria-expanded") === "true";
      navToggle.setAttribute("aria-expanded", String(!expanded));
      navigation.dataset.open = String(!expanded);
    });
  }

  class IslandController {
    constructor(root) {
      this.root = root;
      this.kind = root.dataset.tokimuIsland;
      this.statusState = root.querySelector("[data-island-status-state]");
      this.statusDetail = root.querySelector("[data-island-status-detail]");
      this.activateButton = root.querySelector('[data-island-action="activate"]');
      this.resetButton = root.querySelector('[data-island-action="reset"]');
      this.abortController = null;
      this.release = null;
      this.configError = null;
      this.config = this.readConfig();
      this.handleClick = this.handleClick.bind(this);
      this.root.addEventListener("click", this.handleClick);
      if (this.configError) {
        this.setState("failed", this.configError);
      } else {
        this.setState(root.dataset.state || "idle");
      }
    }

    readConfig() {
      const source = this.root.querySelector(
        'script[type="application/json"][data-island-config]',
      );
      if (!source) {
        return {};
      }

      try {
        return JSON.parse(source.textContent);
      } catch (error) {
        this.configError = "Island configuration is not valid JSON.";
        console.error(`Tokimu island "${this.kind}" configuration failed`, error);
        return {};
      }
    }

    handleClick(event) {
      const action = event.target.closest("[data-island-action]");
      if (!action || !this.root.contains(action)) {
        return;
      }

      if (action.dataset.islandAction === "activate") {
        this.mount();
      } else if (action.dataset.islandAction === "reset") {
        this.reset();
      }
    }

    setState(state, detail) {
      const nextState = ISLAND_STATES.has(state) ? state : "failed";
      this.root.dataset.state = nextState;

      if (this.statusState) {
        this.statusState.textContent = nextState;
      }
      if (this.statusDetail && detail) {
        this.statusDetail.textContent = detail;
      }

      const busy = nextState === "loading";
      if (this.activateButton) {
        this.activateButton.disabled = busy || nextState === "ready";
      }
      if (this.resetButton) {
        this.resetButton.hidden = ![
          "ready",
          "unsupported",
          "failed",
        ].includes(nextState);
      }

      this.root.dispatchEvent(
        new CustomEvent("tokimu:island-state", {
          bubbles: true,
          detail: { kind: this.kind, state: nextState },
        }),
      );
    }

    async mount() {
      if (this.root.dataset.state === "loading") {
        return;
      }

      const loader = loaders.get(this.kind);
      if (!loader) {
        this.setState(
          "unsupported",
          "The Tokimu WASM consumer is not published in this build.",
        );
        return;
      }

      this.abortController = new AbortController();
      this.setState("loading", "Loading the bounded Tokimu consumer...");

      try {
        const mounted = await loader({
          root: this.root,
          config: this.config,
          signal: this.abortController.signal,
        });

        if (this.abortController.signal.aborted) {
          return;
        }

        this.release =
          mounted && typeof mounted.release === "function"
            ? mounted.release
            : null;
        this.setState("ready", "Tokimu evidence consumer ready.");
      } catch (error) {
        if (error && error.name === "AbortError") {
          return;
        }
        if (error && error.name === "NotSupportedError") {
          this.setState(
            "unsupported",
            error.message || "This browser cannot run the Tokimu evidence consumer.",
          );
          return;
        }
        this.setState("failed", "The Tokimu evidence consumer failed to load.");
        console.error(`Tokimu island "${this.kind}" failed`, error);
      }
    }

    unmount() {
      if (this.abortController) {
        this.abortController.abort();
        this.abortController = null;
      }
      if (this.release) {
        this.release();
        this.release = null;
      }
      this.setState("unmounted", "Interactive resources released.");
    }

    reset() {
      this.unmount();
      this.setState("idle", "No engine payload loaded.");
    }

    dispose() {
      this.unmount();
      this.root.removeEventListener("click", this.handleClick);
    }
  }

  function discover() {
    document.querySelectorAll("[data-tokimu-island]").forEach((root) => {
      if (!controllers.has(root)) {
        controllers.set(root, new IslandController(root));
      }
    });
  }

  window.TokimuIslands = Object.freeze({
    register(kind, loader) {
      if (!kind || typeof loader !== "function") {
        throw new TypeError("Tokimu island registration needs a name and loader.");
      }
      loaders.set(kind, loader);
    },
    unregister(kind) {
      loaders.delete(kind);
    },
    discover,
  });

  discover();

  window.addEventListener(
    "pagehide",
    () => {
      controllers.forEach((controller) => controller.dispose());
      controllers.clear();
    },
    { once: true },
  );
})();
