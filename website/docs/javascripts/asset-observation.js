"use strict";
const scriptUrl = new URL(document.currentScript?.src ?? window.location.href);
const defaultEngineUrl = new URL("../assets/islands/asset-observation/tokimu_asset_workbench_engine.js", scriptUrl);
const defaultFixtureUrl = new URL("../assets/islands/asset-observation/shapes-rect-01-geometry.svg", scriptUrl);
const MAX_DIAGNOSTICS = 8;
let workbenchSequence = 0;
window.TokimuIslands.register("asset-observation", async ({ root, config: rawConfig, signal }) => {
    if (typeof WebAssembly === "undefined") {
        throw new DOMException("This browser does not provide WebAssembly.", "NotSupportedError");
    }
    const config = rawConfig;
    const mount = required(root, "[data-island-mount]");
    const fallback = required(root, ".island-fallback");
    const engineUrl = new URL(config.engineModuleUrl ?? defaultEngineUrl.href, scriptUrl);
    const fixtureUrl = new URL(config.fixtureUrl ?? defaultFixtureUrl.href, scriptUrl);
    const maxBytes = boundedMaxBytes(config.maxBytes);
    const startedAt = performance.now();
    const engine = (await import(engineUrl.href));
    await engine.default();
    throwIfAborted(signal);
    const wasmStartupMs = performance.now() - startedAt;
    const workbench = createWorkbench();
    const canvas = required(workbench, "canvas");
    const fileInput = required(workbench, 'input[type="file"]');
    const loadFixture = required(workbench, '[data-evidence-action="fixture"]');
    const context = canvas.getContext("2d");
    if (!context) {
        throw new DOMException("This browser does not provide Canvas 2D.", "NotSupportedError");
    }
    mount.replaceChildren(workbench);
    mount.hidden = false;
    fallback.hidden = true;
    let observation = null;
    let resizeObserver = null;
    let intersectionObserver = null;
    let isIntersecting = true;
    let firstEvidenceMs = null;
    const drawCurrent = () => {
        if (document.hidden || !isIntersecting)
            return false;
        if (observation)
            drawObservation(canvas, context, observation);
        else
            drawEmpty(canvas, context);
        return true;
    };
    const presentBytes = (fileName, bytes) => {
        if (bytes.byteLength > maxBytes) {
            presentFailure(mount, fileName, `Input is ${formatBytes(bytes.byteLength)}; this island accepts at most ${formatBytes(maxBytes)}.`);
            observation = null;
            drawCurrent();
            return;
        }
        try {
            const inspectionStartedAt = performance.now();
            observation = JSON.parse(engine.inspect_asset(fileName, bytes));
            const inspectionMs = performance.now() - inspectionStartedAt;
            presentObservation(mount, observation, engine.engine_status());
            const presentationStartedAt = performance.now();
            const canvasPresented = drawCurrent();
            const canvasPresentationMs = canvasPresented
                ? performance.now() - presentationStartedAt
                : null;
            firstEvidenceMs ??= performance.now() - startedAt;
            presentTiming(mount, {
                wasmStartupMs,
                inspectionMs,
                firstEvidenceMs,
                canvasPresentationMs,
            });
        }
        catch (error) {
            observation = null;
            presentFailure(mount, fileName, message(error));
            drawCurrent();
        }
    };
    const loadKnownFixture = async () => {
        setLoadingMessage(mount, "Loading the repository-owned W3C fixture...");
        const response = await fetch(fixtureUrl, { signal });
        if (!response.ok) {
            throw new Error(`Known fixture request failed with HTTP ${response.status}.`);
        }
        const declaredLength = Number(response.headers.get("content-length"));
        if (Number.isFinite(declaredLength) && declaredLength > maxBytes) {
            throw new Error(`Known fixture is ${formatBytes(declaredLength)}; this island accepts at most ${formatBytes(maxBytes)}.`);
        }
        const bytes = new Uint8Array(await response.arrayBuffer());
        throwIfAborted(signal);
        presentBytes(config.fixtureName ?? "shapes-rect-01-geometry.svg", bytes);
    };
    const onFixture = () => void loadKnownFixture().catch((error) => {
        if (error instanceof DOMException && error.name === "AbortError")
            return;
        presentFailure(mount, "Known fixture", message(error));
    });
    const onFile = () => {
        const file = fileInput.files?.[0];
        if (!file)
            return;
        if (file.size > maxBytes) {
            presentFailure(mount, file.name, `Input is ${formatBytes(file.size)}; this island accepts at most ${formatBytes(maxBytes)}.`);
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
        drawCurrent();
    };
    const onVisibility = () => {
        if (!document.hidden)
            drawCurrent();
    };
    loadFixture.addEventListener("click", onFixture);
    fileInput.addEventListener("change", onFile);
    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);
    if ("ResizeObserver" in window) {
        resizeObserver = new ResizeObserver(onResize);
        resizeObserver.observe(canvas);
    }
    if ("IntersectionObserver" in window) {
        intersectionObserver = new IntersectionObserver((entries) => {
            isIntersecting = entries.some((entry) => entry.isIntersecting);
            if (isIntersecting)
                drawCurrent();
        });
        intersectionObserver.observe(workbench);
    }
    const release = () => {
        resizeObserver?.disconnect();
        intersectionObserver?.disconnect();
        window.removeEventListener("resize", onResize);
        document.removeEventListener("visibilitychange", onVisibility);
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
    }
    catch (error) {
        if (error instanceof DOMException && error.name === "AbortError") {
            release();
            throw error;
        }
        presentFailure(mount, "Known fixture", message(error));
    }
    return { release };
});
function createWorkbench() {
    workbenchSequence += 1;
    const nameId = `asset-evidence-name-${workbenchSequence}`;
    const summaryId = `asset-evidence-summary-${workbenchSequence}`;
    const reportId = `asset-evidence-report-${workbenchSequence}`;
    const workbench = document.createElement("div");
    workbench.className = "asset-evidence";
    workbench.innerHTML = `
    <div class="asset-evidence-preview">
      <div class="asset-evidence-preview-header">
        <span>Tokimu vector preview</span>
        <strong data-evidence-badge>Loading</strong>
      </div>
      <canvas
        role="img"
        aria-label="Tokimu-rendered diagnostic preview of the selected SVG"
        aria-describedby="${summaryId} ${reportId}"
      >
        The visual preview requires Canvas 2D. The adjacent report contains the
        authoritative observation as text.
      </canvas>
    </div>
    <div
      class="asset-evidence-report"
      id="${reportId}"
      role="region"
      aria-labelledby="${nameId}"
    >
      <p class="eyebrow">Local WASM observation</p>
      <h3 id="${nameId}" data-evidence-name>Preparing fixture</h3>
      <p id="${summaryId}" data-evidence-summary>Starting the bounded Tokimu consumer...</p>
      <p
        class="visually-hidden"
        role="status"
        aria-live="polite"
        aria-atomic="true"
        data-evidence-announcement
      >Tokimu evidence consumer is starting.</p>
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
function presentObservation(mount, observation, engineStatus) {
    text(mount, "[data-evidence-name]", observation.fileName);
    text(mount, "[data-evidence-summary]", observation.summary);
    text(mount, "[data-evidence-badge]", `${observation.format} / ${observation.status}`.toUpperCase());
    const properties = required(mount, "[data-evidence-properties]");
    properties.replaceChildren();
    appendProperty(properties, "Bytes", observation.byteLength.toLocaleString());
    for (const property of observation.properties.slice(0, 6)) {
        appendProperty(properties, property.label, property.value);
    }
    const paths = observation.preview?.paths ?? [];
    const contours = paths.reduce((total, path) => total + path.contours.length, 0);
    const points = paths.reduce((total, path) => total + path.contours.reduce((pathTotal, contour) => pathTotal + contour.points.length, 0), 0);
    appendProperty(properties, "Vector records", paths.length.toString());
    appendProperty(properties, "Contours", contours.toString());
    appendProperty(properties, "Flattened points", points.toString());
    const knownFixtureMatches = observation.status === "renderable" &&
        observation.fileName === "shapes-rect-01-geometry.svg" &&
        paths.length === 4 &&
        contours === 4;
    const verdict = required(mount, "[data-evidence-verdict]");
    if (observation.status === "error") {
        verdict.dataset.result = "failed";
        verdict.textContent = "Tokimu rejected the input; no rendering claim was made.";
    }
    else {
        verdict.dataset.result = knownFixtureMatches ? "pass" : "observed";
        verdict.textContent = knownFixtureMatches
            ? "Expected evidence matched: 4 vector records and 4 contours."
            : "Observation complete. Expected-fixture assertions apply only to the repository fixture.";
    }
    verdict.title = engineStatus;
    const diagnostics = required(mount, "[data-evidence-diagnostics]");
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
    text(mount, "[data-evidence-announcement]", `${observation.fileName}: ${observation.summary} ${verdict.textContent ?? ""}`);
}
function presentTiming(mount, timing) {
    const properties = required(mount, "[data-evidence-properties]");
    appendProperty(properties, "WASM startup", formatMilliseconds(timing.wasmStartupMs));
    appendProperty(properties, "Inspection", formatMilliseconds(timing.inspectionMs));
    appendProperty(properties, "First evidence", formatMilliseconds(timing.firstEvidenceMs));
    appendProperty(properties, "Canvas presentation", timing.canvasPresentationMs === null
        ? "Deferred while hidden or offscreen"
        : formatMilliseconds(timing.canvasPresentationMs));
}
function presentFailure(mount, fileName, detail) {
    text(mount, "[data-evidence-name]", fileName);
    text(mount, "[data-evidence-summary]", "Tokimu rejected this source input.");
    text(mount, "[data-evidence-badge]", "SVG / ERROR");
    const properties = required(mount, "[data-evidence-properties]");
    properties.replaceChildren();
    const verdict = required(mount, "[data-evidence-verdict]");
    verdict.dataset.result = "failed";
    verdict.textContent = "No rendering claim was made for this input.";
    const diagnostics = required(mount, "[data-evidence-diagnostics]");
    diagnostics.replaceChildren();
    const item = document.createElement("li");
    item.textContent = detail;
    diagnostics.append(item);
    text(mount, "[data-evidence-announcement]", `${fileName}: Tokimu rejected this source input. ${detail}`);
}
function setLoadingMessage(mount, detail) {
    text(mount, "[data-evidence-summary]", detail);
    text(mount, "[data-evidence-badge]", "Loading");
}
function drawObservation(canvas, context, observation) {
    resizeCanvas(canvas, context);
    drawEmpty(canvas, context);
    const paths = observation.preview?.paths ?? [];
    const points = paths.flatMap((path) => path.contours.flatMap((contour) => contour.points));
    if (!points.length)
        return;
    const xs = points.map(([x]) => x);
    const ys = points.map(([, y]) => y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const width = Math.max(maxX - minX, 0.001);
    const height = Math.max(maxY - minY, 0.001);
    const padding = 54 * (window.devicePixelRatio || 1);
    const scale = Math.min((canvas.width - padding * 2) / width, (canvas.height - padding * 2) / height);
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
                if (index === 0)
                    context.moveTo(px, py);
                else
                    context.lineTo(px, py);
            });
            if (contour.closed)
                context.closePath();
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
function resizeCanvas(canvas, context) {
    const ratio = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = Math.max(320, Math.min(rect.width || 640, 900));
    const height = Math.max(260, Math.min(rect.height || 420, 600));
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(height * ratio);
    context.setTransform(1, 0, 0, 1, 0, 0);
}
function drawEmpty(canvas, context) {
    context.fillStyle = "#080d0f";
    context.fillRect(0, 0, canvas.width, canvas.height);
}
function appendProperty(list, label, value) {
    const term = document.createElement("dt");
    term.textContent = label;
    const detail = document.createElement("dd");
    detail.textContent = value;
    list.append(term, detail);
}
function boundedMaxBytes(value) {
    if (!Number.isFinite(value))
        return 8 * 1024 * 1024;
    return Math.max(1, Math.min(Number(value), 64 * 1024 * 1024));
}
function throwIfAborted(signal) {
    if (signal.aborted) {
        throw new DOMException("Island activation was cancelled.", "AbortError");
    }
}
function formatBytes(value) {
    return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}
function formatMilliseconds(value) {
    return `${value.toFixed(1)} ms observed`;
}
function text(root, selector, value) {
    required(root, selector).textContent = value;
}
function required(root, selector) {
    const element = root.querySelector(selector);
    if (!element)
        throw new Error(`Missing evidence element ${selector}.`);
    return element;
}
function message(error) {
    return error instanceof Error ? error.message : String(error);
}
