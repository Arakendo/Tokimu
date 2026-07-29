import { createServer } from "node:http";
import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = fileURLToPath(new URL(".", import.meta.url));
const workspaceRoot = resolve(rootDir, "..", "..", "..");
const port = Number.parseInt(process.env.PORT ?? "4173", 10);

const contentTypes = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".mjs", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
  [".css", "text/css; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".svg", "image/svg+xml"],
  [".png", "image/png"],
  [".jpg", "image/jpeg"],
  [".jpeg", "image/jpeg"],
  [".ico", "image/x-icon"],
]);

const wasmPath = resolve(workspaceRoot, "target/wasm32-unknown-unknown/debug/hello-fps-web.wasm");

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    shell: true,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

run("cargo", ["build", "--target", "wasm32-unknown-unknown", "-p", "hello-fps-web", "--bin", "hello-fps-web"], workspaceRoot);
run("wasm-bindgen", [wasmPath, "--out-dir", "pkg", "--target", "web"], rootDir);

const server = createServer(async (request, response) => {
  const requestUrl = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
  const relativePath = requestUrl.pathname === "/" ? "/index.html" : requestUrl.pathname;
  const safePath = normalize(relativePath).replace(/^([.]{2}[\/\\])+/, "");
  const filePath = join(rootDir, safePath);

  try {
    const content = await readFile(filePath);
    response.statusCode = 200;
    response.setHeader("Content-Type", contentTypes.get(extname(filePath)) ?? "application/octet-stream");
    response.end(content);
  } catch {
    response.statusCode = 404;
    response.setHeader("Content-Type", "text/plain; charset=utf-8");
    response.end("Not found");
  }
});

server.listen(port, "127.0.0.1", () => {
  console.log(`Tokimu hello-fps-web server running at http://127.0.0.1:${port}`);
});