import init, {
  terminal_fixture_cpu_height,
  terminal_fixture_cpu_rgba,
  terminal_fixture_cpu_summary,
  terminal_fixture_cpu_width,
} from "./hello_terminal_surface.js";

const canvas = document.querySelector("#surface");
const status = document.querySelector("#status");
const requestedProducer = new URLSearchParams(window.location.search).get("producer");
const producer = requestedProducer === "ratatui" ? "ratatui" : "independent";

try {
  await init();
  const width = terminal_fixture_cpu_width(producer);
  const height = terminal_fixture_cpu_height(producer);
  const rgba = terminal_fixture_cpu_rgba(producer);
  const context = canvas.getContext("2d", { alpha: false });
  canvas.width = width;
  canvas.height = height;
  context.putImageData(new ImageData(new Uint8ClampedArray(rgba), width, height), 0, 0);
  status.textContent = `Browser presentation complete. producer=${producer}\n${terminal_fixture_cpu_summary(producer)}`;
} catch (error) {
  status.textContent = `Browser evidence failed: ${error}`;
  console.error(error);
}
