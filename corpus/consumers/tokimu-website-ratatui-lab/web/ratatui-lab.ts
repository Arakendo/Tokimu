import init, {
  cell_pixel_height,
  cell_pixel_width,
  template_catalog_json,
  template_frame_rgba,
} from "./tokimu_website_ratatui_lab_engine.js";

type Template = { id: string; label: string; description: string };
const templateSelect = need<HTMLSelectElement>("[data-template]");
const densitySelect = need<HTMLSelectElement>("[data-density]");
const canvas = need<HTMLCanvasElement>("[data-terminal]");
const status = need<HTMLElement>("[data-status]");
const description = need<HTMLElement>("[data-description]");
const summary = need<HTMLElement>("[data-summary]");
const gridSize = need<HTMLOutputElement>("[data-grid-size]");
const context = canvas.getContext("2d");

if (!context) throw new Error("Ratatui lab requires a 2D canvas context.");
const drawingContext = context;

let templates: Template[] = [];

await init();
templates = read<Template[]>(template_catalog_json());
templateSelect.replaceChildren(...templates.map(templateOption));
render();

templateSelect.addEventListener("change", render);
densitySelect.addEventListener("change", render);

function render(): void {
  const [width, height] = densitySelect.value.split("x").map(Number);
  const active = templates.find((template) => template.id === templateSelect.value) ?? templates[0];
  const rgba = template_frame_rgba(active.id, width, height);
  drawFrame(rgba, width, height);
  description.textContent = active.description;
  gridSize.value = `${width} x ${height} cells`;
  summary.textContent = [
    `template: ${active.label}`,
    `cells: ${width * height}`,
    `pixels: ${width * cell_pixel_width()} x ${height * cell_pixel_height()}`,
    "composition: Ratatui widgets -> TokimuBackend -> Tokimu text raster",
    "browser role: fixture selection and RGBA frame blit",
  ].join("\n");
  status.textContent = `Tokimu-rendered Rust/WASM frame / ${active.label}`;
}

function drawFrame(rgba: Uint8Array, columns: number, rows: number): void {
  const width = columns * cell_pixel_width();
  const height = rows * cell_pixel_height();
  const expectedBytes = width * height * 4;
  if (rgba.length !== expectedBytes) {
    throw new Error(`Tokimu frame returned ${rgba.length} bytes; expected ${expectedBytes}.`);
  }
  canvas.width = width;
  canvas.height = height;
  canvas.style.aspectRatio = `${width} / ${height}`;
  const pixels = new Uint8ClampedArray(rgba);
  drawingContext.putImageData(new ImageData(pixels, width, height), 0, 0);
}

function templateOption(template: Template): HTMLOptionElement {
  const option = document.createElement("option");
  option.value = template.id;
  option.textContent = template.label;
  return option;
}

function read<T>(json: string): T {
  return JSON.parse(json) as T;
}

function need<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Ratatui lab requires ${selector}.`);
  return element;
}
