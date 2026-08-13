import { createServer } from "node:http";
import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const webRoot = fileURLToPath(new URL(".", import.meta.url));
const workspaceRoot = resolve(webRoot, "..", "..", "..");
const port = Number.parseInt(process.env.PORT ?? "4174", 10);
const wasmPath = resolve(
  workspaceRoot,
  "target/wasm32-unknown-unknown/debug/hello-render-orientation-web.wasm",
);

const contentTypes = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
  [".css", "text/css; charset=utf-8"],
]);

function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell: true });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

if (process.env.TOKIMU_SKIP_WASM_BUILD !== "1") {
  run(
    "cargo",
    ["build", "--target", "wasm32-unknown-unknown", "-p", "hello-render-orientation-web"],
    workspaceRoot,
  );
  run(
    "wasm-bindgen",
    [wasmPath, "--out-dir", "pkg", "--target", "web"],
    webRoot,
  );
}

const server = createServer(async (request, response) => {
  const requestUrl = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
  const relativePath = requestUrl.pathname === "/"
    ? "index.html"
    : requestUrl.pathname.replace(/^\/+/, "");
  const safePath = normalize(relativePath).replace(/^([.]{2}[\\/])+/, "");
  try {
    const content = await readFile(join(webRoot, safePath));
    response.statusCode = 200;
    response.setHeader("Content-Type", contentTypes.get(extname(safePath)) ?? "application/octet-stream");
    response.setHeader("Cache-Control", "no-store");
    response.end(content);
  } catch {
    response.statusCode = 404;
    response.end("Not found");
  }
});

server.listen(port, "127.0.0.1", () => {
  console.log(`Tokimu orientation fixture at http://127.0.0.1:${port}`);
});
