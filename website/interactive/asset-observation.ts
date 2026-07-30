type PreviewContour = {
  points: [number, number][];
  closed: boolean;
};

type PreviewPath = {
  contours: PreviewContour[];
  fill: boolean;
  stroke: boolean;
  color: [number, number, number, number];
  stroke_width: number;
};

type AssetObservation = {
  schema: number;
  fileName: string;
  format: string;
  status: string;
  byteLength: number;
  summary: string;
  properties: { label: string; value: string }[];
  diagnostics: string[];
  preview: {
    kind: string;
    paths: PreviewPath[];
  } | null;
};

type IslandConfig = {
  fixture?: string;
  fixtureUrl?: string;
  fixtureName?: string;
  engineModuleUrl?: string;
  maxBytes?: number;
};

type EngineModule = {
  default(moduleOrPath?: RequestInfo | URL): Promise<unknown>;
  engine_status(): string;
  inspect_asset(fileName: string, bytes: Uint8Array): string;
};

const scriptUrl = new URL(
  (document.currentScript as HTMLScriptElement | null)?.src ?? window.location.href,
);
const defaultEngineUrl = new URL(
  "../assets/islands/asset-observation/tokimu_asset_workbench_engine.js",
  scriptUrl,
);
const defaultFixtureUrl = new URL(
  "../assets/islands/asset-observation/shapes-rect-01-geometry.svg",
  scriptUrl,
);
const MAX_DIAGNOSTICS = 8;

window.TokimuIslands.register(
  "asset-observation",
  async ({ root, config: rawConfig, signal }) => {
    if (typeof WebAssembly === "undefined") {
      throw new DOMException(
        "This browser does not provide WebAssembly.",
        "NotSupportedError",
      );
    }

    const config = rawConfig as IslandConfig;
    const mount = required<HTMLElement>(root, "[data-island-mount]");
    const fallback = required<HTMLElement>(root, ".island-fallback");
    const engineUrl = new URL(config.engineModuleUrl ?? defaultEngineUrl.href, scriptUrl);
    const fixtureUrl = new URL(config.fixtureUrl ?? defaultFixtureUrl.href, scriptUrl);
    const maxBytes = boundedMaxBytes(config.maxBytes);
    const startedAt = performance.now();

    const engine = (await import(engineUrl.href)) as EngineModule;
    await engine.default();
    throwIfAborted(signal);

    const workbench = createWorkbench();
    const canvas = required<HTMLCanvasElement>(workbench, "canvas");
    const fileInput = required<HTMLInputElement>(workbench, 'input[type="file"]');
    const loadFixture = required<HTMLButtonElement>(
      workbench,
      '[data-evidence-action="fixture"]',
    );
    const context = canvas.getContext("2d");
    if (!context) {
      throw new DOMException(
        "This browser does not provide Canvas 2D.",
        "NotSupportedError",
      );
    }

    mount.replaceChildren(workbench);
    mount.hidden = false;
    fallback.hidden = true;

    let observation: AssetObservation | null = null;
    let resizeObserver: ResizeObserver | null = null;

    const presentBytes = (fileName: string, bytes: Uint8Array) => {
      if (bytes.byteLength > maxBytes) {
        presentFailure(
          mount,
          fileName,
          `Input is ${formatBytes(bytes.byteLength)}; this island accepts at most ${formatBytes(maxBytes)}.`,
        );
        drawEmpty(canvas, context);
        return;
      }

      try {
        observation = JSON.parse(engine.inspect_asset(fileName, bytes)) as AssetObservation;
        presentObservation(mount, observation, engine.engine_status(), startedAt);
        drawObservation(canvas, context, observation);
      } catch (error) {
        observation = null;
        presentFailure(mount, fileName, message(error));
        drawEmpty(canvas, context);
      }
    };

    const loadKnownFixture = async () => {
      setLoadingMessage(mount, "Loading the repository-owned W3C fixture...");
      const response = await fetch(fixtureUrl, { signal });
      if (!response.ok) {
        throw new Error(`Known fixture request failed with HTTP ${response.status}.`);
      }
      const bytes = new Uint8Array(await response.arrayBuffer());
      throwIfAborted(signal);
      presentBytes(config.fixtureName ?? "shapes-rect-01-geometry.svg", bytes);
    };

    const onFixture = () => void loadKnownFixture().catch((error) => {
      if (error instanceof DOMException && error.name === "AbortError") return;
      presentFailure(mount, "Known fixture", message(error));
    });
    const onFile = () => {
      const file = fileInput.files?.[0];
      if (!file) return;
      if (file.size > maxBytes) {
        presentFailure(
          mount,
          file.name,
          `Input is ${formatBytes(file.size)}; this island accepts at most ${formatBytes(maxBytes)}.`,
        );
        return;
      }
      void file.arrayBuffer()
        .then((buffer) => {
          if (!signal.aborted) {
            presentBytes(file.name, new Uint8Array(buffer));
          }
        })
        .catch((error) => {
          if (!signal.aborted) {
            presentFailure(mount, file.name, message(error));
          }
        });
    };
    const onResize = () => {
      if (observation) drawObservation(canvas, context, observation);
      else drawEmpty(canvas, context);
    };

    loadFixture.addEventListener("click", onFixture);
    fileInput.addEventListener("change", onFile);
    window.addEventListener("resize", onResize);
    if ("ResizeObserver" in window) {
      resizeObserver = new ResizeObserver(onResize);
      resizeObserver.observe(canvas);
    }

    const release = () => {
      resizeObserver?.disconnect();
      window.removeEventListener("resize", onResize);
      loadFixture.removeEventListener("click", onFixture);
      fileInput.removeEventListener("change", onFile);
      fileInput.value = "";
      observation = null;
      mount.replaceChildren();
      mount.hidden = true;
      fallback.hidden = false;
    };

    try {
      await loadKnownFixture();
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        release();
        throw error;
      }
      presentFailure(mount, "Known fixture", message(error));
    }

    return { release };
  },
);

function createWorkbench(): HTMLElement {
  const workbench = document.createElement("div");
  workbench.className = "asset-evidence";
  workbench.innerHTML = `
    <div class="asset-evidence-preview">
      <div class="asset-evidence-preview-header">
        <span>Tokimu vector preview</span>
        <strong data-evidence-badge>Loading</strong>
      </div>
      <canvas aria-label="Tokimu-rendered diagnostic preview of the selected SVG"></canvas>
    </div>
    <div class="asset-evidence-report">
      <p class="eyebrow">Local WASM observation</p>
      <h3 data-evidence-name>Preparing fixture</h3>
      <p data-evidence-summary>Starting the bounded Tokimu consumer...</p>
      <dl data-evidence-properties></dl>
      <div class="asset-evidence-verdict" data-evidence-verdict>
        Waiting for structural evidence.
      </div>
      <div class="asset-evidence-diagnostics">
        <strong>Diagnostics</strong>
        <ul data-evidence-diagnostics></ul>
      </div>
      <div class="asset-evidence-controls">
        <button class="button button-secondary" type="button" data-evidence-action="fixture">
          Reload known fixture
        </button>
        <label class="button button-secondary">
          Inspect local SVG
          <input type="file" accept=".svg,image/svg+xml">
        </label>
      </div>
      <p class="asset-evidence-privacy">
        Files stay in this browser tab. This page does not upload selected bytes.
      </p>
    </div>
  `;
  return workbench;
}

function presentObservation(
  mount: HTMLElement,
  observation: AssetObservation,
  engineStatus: string,
  startedAt: number,
): void {
  text(mount, "[data-evidence-name]", observation.fileName);
  text(mount, "[data-evidence-summary]", observation.summary);
  text(mount, "[data-evidence-badge]", `${observation.format} / ${observation.status}`.toUpperCase());

  const properties = required<HTMLDListElement>(mount, "[data-evidence-properties]");
  properties.replaceChildren();
  appendProperty(properties, "Bytes", observation.byteLength.toLocaleString());
  for (const property of observation.properties.slice(0, 6)) {
    appendProperty(properties, property.label, property.value);
  }

  const paths = observation.preview?.paths ?? [];
  const contours = paths.reduce((total, path) => total + path.contours.length, 0);
  const points = paths.reduce(
    (total, path) =>
      total + path.contours.reduce((pathTotal, contour) => pathTotal + contour.points.length, 0),
    0,
  );
  appendProperty(properties, "Vector records", paths.length.toString());
  appendProperty(properties, "Contours", contours.toString());
  appendProperty(properties, "Flattened points", points.toString());
  appendProperty(properties, "Startup", `${(performance.now() - startedAt).toFixed(1)} ms`);

  const knownFixtureMatches =
    observation.status === "renderable" &&
    observation.fileName === "shapes-rect-01-geometry.svg" &&
    paths.length === 4 &&
    contours === 4;
  const verdict = required<HTMLElement>(mount, "[data-evidence-verdict]");
  if (observation.status === "error") {
    verdict.dataset.result = "failed";
    verdict.textContent = "Tokimu rejected the input; no rendering claim was made.";
  } else {
    verdict.dataset.result = knownFixtureMatches ? "pass" : "observed";
    verdict.textContent = knownFixtureMatches
      ? "Expected evidence matched: 4 vector records and 4 contours."
      : "Observation complete. Expected-fixture assertions apply only to the repository fixture.";
  }
  verdict.title = engineStatus;

  const diagnostics = required<HTMLUListElement>(mount, "[data-evidence-diagnostics]");
  diagnostics.replaceChildren();
  const messages = observation.diagnostics.length
    ? observation.diagnostics.slice(0, MAX_DIAGNOSTICS)
    : ["No importer diagnostics were emitted."];
  for (const diagnostic of messages) {
    const item = document.createElement("li");
    item.textContent = diagnostic;
    diagnostics.append(item);
  }
  if (observation.diagnostics.length > MAX_DIAGNOSTICS) {
    const item = document.createElement("li");
    item.textContent = `${observation.diagnostics.length - MAX_DIAGNOSTICS} additional diagnostics omitted.`;
    diagnostics.append(item);
  }
}

function presentFailure(mount: HTMLElement, fileName: string, detail: string): void {
  text(mount, "[data-evidence-name]", fileName);
  text(mount, "[data-evidence-summary]", "Tokimu rejected this source input.");
  text(mount, "[data-evidence-badge]", "SVG / ERROR");
  const properties = required<HTMLElement>(mount, "[data-evidence-properties]");
  properties.replaceChildren();
  const verdict = required<HTMLElement>(mount, "[data-evidence-verdict]");
  verdict.dataset.result = "failed";
  verdict.textContent = "No rendering claim was made for this input.";
  const diagnostics = required<HTMLUListElement>(mount, "[data-evidence-diagnostics]");
  diagnostics.replaceChildren();
  const item = document.createElement("li");
  item.textContent = detail;
  diagnostics.append(item);
}

function setLoadingMessage(mount: HTMLElement, detail: string): void {
  text(mount, "[data-evidence-summary]", detail);
  text(mount, "[data-evidence-badge]", "Loading");
}

function drawObservation(
  canvas: HTMLCanvasElement,
  context: CanvasRenderingContext2D,
  observation: AssetObservation,
): void {
  resizeCanvas(canvas, context);
  drawEmpty(canvas, context);
  const paths = observation.preview?.paths ?? [];
  const points = paths.flatMap((path) =>
    path.contours.flatMap((contour) => contour.points),
  );
  if (!points.length) return;

  const xs = points.map(([x]) => x);
  const ys = points.map(([, y]) => y);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const width = Math.max(maxX - minX, 0.001);
  const height = Math.max(maxY - minY, 0.001);
  const padding = 54 * (window.devicePixelRatio || 1);
  const scale = Math.min(
    (canvas.width - padding * 2) / width,
    (canvas.height - padding * 2) / height,
  );
  const offsetX = (canvas.width - width * scale) / 2;
  const offsetY = (canvas.height - height * scale) / 2;

  context.lineJoin = "round";
  context.lineCap = "round";
  for (const path of paths) {
    context.beginPath();
    for (const contour of path.contours) {
      contour.points.forEach(([x, y], index) => {
        const px = offsetX + (x - minX) * scale;
        const py = offsetY + (y - minY) * scale;
        if (index === 0) context.moveTo(px, py);
        else context.lineTo(px, py);
      });
      if (contour.closed) context.closePath();
    }
    const [red, green, blue, alpha] = path.color;
    const color = `rgba(${red * 255}, ${green * 255}, ${blue * 255}, ${alpha})`;
    if (path.fill) {
      context.fillStyle = color;
      context.fill("evenodd");
    }
    if (path.stroke) {
      context.strokeStyle = color;
      context.lineWidth = Math.max(1.5, path.stroke_width * scale);
      context.stroke();
    }
  }
}

function resizeCanvas(
  canvas: HTMLCanvasElement,
  context: CanvasRenderingContext2D,
): void {
  const ratio = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(320, Math.min(rect.width || 640, 900));
  const height = Math.max(260, Math.min(rect.height || 420, 600));
  canvas.width = Math.round(width * ratio);
  canvas.height = Math.round(height * ratio);
  context.setTransform(1, 0, 0, 1, 0, 0);
}

function drawEmpty(
  canvas: HTMLCanvasElement,
  context: CanvasRenderingContext2D,
): void {
  context.fillStyle = "#080d0f";
  context.fillRect(0, 0, canvas.width, canvas.height);
}

function appendProperty(list: HTMLDListElement, label: string, value: string): void {
  const term = document.createElement("dt");
  term.textContent = label;
  const detail = document.createElement("dd");
  detail.textContent = value;
  list.append(term, detail);
}

function boundedMaxBytes(value: number | undefined): number {
  if (!Number.isFinite(value)) return 8 * 1024 * 1024;
  return Math.max(1, Math.min(Number(value), 64 * 1024 * 1024));
}

function throwIfAborted(signal: AbortSignal): void {
  if (signal.aborted) {
    throw new DOMException("Island activation was cancelled.", "AbortError");
  }
}

function formatBytes(value: number): string {
  return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}

function text(root: ParentNode, selector: string, value: string): void {
  required<HTMLElement>(root, selector).textContent = value;
}

function required<T extends Element>(root: ParentNode, selector: string): T {
  const element = root.querySelector<T>(selector);
  if (!element) throw new Error(`Missing evidence element ${selector}.`);
  return element;
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
