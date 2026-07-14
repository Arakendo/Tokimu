import init from "./pkg/tokimu_wasm.js";

const status = document.getElementById("status");

function setStatus(message) {
  if (status) {
    status.textContent = message;
  }
}

async function main() {
  setStatus("Initializing Tokimu wasm bootstrap...");
  await init();
  setStatus("Tokimu wasm ready");
}

main().catch((error) => {
  console.error(error);
  setStatus(`Tokimu wasm bootstrap failed: ${error}`);
});
