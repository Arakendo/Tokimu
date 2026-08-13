import { createServer } from "node:http";
import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL(".", import.meta.url));
const workspace = resolve(root, "..", "..", "..");
const port = Number.parseInt(process.env.PORT ?? "4186", 10);
const wasm = resolve(workspace, "target/wasm32-unknown-unknown/debug/hello-bulk-compute-web.wasm");
const contentTypes = new Map([[".html", "text/html; charset=utf-8"], [".js", "text/javascript; charset=utf-8"], [".wasm", "application/wasm"]]);
const run = (command, args, cwd) => {
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell: true });
  if (result.status !== 0) process.exit(result.status ?? 1);
};
if (process.env.TOKIMU_SKIP_WASM_BUILD !== "1") {
  run("cargo", ["build", "--target", "wasm32-unknown-unknown", "-p", "hello-bulk-compute-web"], workspace);
  run("wasm-bindgen", [wasm, "--out-dir", "pkg", "--out-name", "hello_bulk_compute_web", "--target", "web"], root);
}
createServer(async (request, response) => {
  const requestUrl = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
  const relative = requestUrl.pathname === "/" ? "index.html" : requestUrl.pathname.replace(/^\/+/, "");
  const safe = normalize(relative).replace(/^([.]{2}[\\/])+/, "");
  try {
    const body = await readFile(join(root, safe));
    response.writeHead(200, { "Content-Type": contentTypes.get(extname(safe)) ?? "application/octet-stream", "Cache-Control": "no-store" });
    response.end(body);
  } catch {
    response.writeHead(404).end("Not found");
  }
}).listen(port, "127.0.0.1", () => console.log(`Tokimu Slice 9 browser control at http://127.0.0.1:${port}`));
