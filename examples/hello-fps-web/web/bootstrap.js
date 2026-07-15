import init from "./pkg/hello-fps-web.js";

async function main() {
  await init();
}

main().catch((error) => {
  console.error(error);
  const status = document.getElementById("status");
  if (status) {
    status.textContent = `Tokimu hello-fps-web wasm bootstrap failed: ${error}`;
  }
});