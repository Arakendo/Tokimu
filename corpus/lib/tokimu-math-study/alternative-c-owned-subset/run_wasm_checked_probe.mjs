// Plain Node WebAssembly-engine runner for the bounded C0 checked probe.
// This intentionally performs no browser, renderer, or JavaScript math work.
import { readFile } from "node:fs/promises";

const wasmPath = process.argv[2];
if (!wasmPath || process.argv.length !== 3) {
  throw new Error("usage: node run_wasm_checked_probe.mjs <candidate.wasm>");
}

const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes);
const observed = instance.exports.tokimu_math_study_wasm_checked_probe();
const expected = 0b11_1111;
if (observed !== expected) {
  throw new Error(`expected checked probe ${expected}, observed ${observed}`);
}
console.log(`option-c wasm checked probe: ${observed}`);

const layout = {
  vec4Size: instance.exports.tokimu_math_study_wasm_vec4_size(),
  vec4Alignment: instance.exports.tokimu_math_study_wasm_vec4_alignment(),
  mat4Size: instance.exports.tokimu_math_study_wasm_mat4_size(),
  mat4Alignment: instance.exports.tokimu_math_study_wasm_mat4_alignment(),
};
console.log(`option-c wasm scalar layout: ${JSON.stringify(layout)}`);
