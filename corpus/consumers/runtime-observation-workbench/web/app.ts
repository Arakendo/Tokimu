import init, { WasmRuntimeObservationSession } from "../dist/runtime_observation_workbench_engine.js";

const output = document.querySelector<HTMLPreElement>("#output");
if (!output) throw new Error("runtime observation output is missing");

await init();
const runtime = new WasmRuntimeObservationSession();
let commandId = 1;
let tick = 1;

const show = (value: string) => {
  output.textContent = JSON.stringify(JSON.parse(value), null, 2);
};

const showError = (error: unknown) => {
  output.textContent = JSON.stringify({ error: String(error) }, null, 2);
};

const on = (id: string, action: () => string) =>
  document.querySelector<HTMLButtonElement>(`#${id}`)?.addEventListener("click", () => {
    try {
      show(action());
    } catch (error) {
      showError(error);
    }
  });

on("observe", () => runtime.observation_json(commandId++, 7));
on("observe-missing", () => runtime.observation_json(commandId++, 99));
on("move", () => runtime.enqueue_json(JSON.stringify({ id: commandId++, target: 7, authority: "operator", command: { command: "move_by", delta: { x: 0.25, y: 0, z: 0 } } })));
on("queue-stale", () => runtime.enqueue_json(JSON.stringify({ id: commandId++, target: 7, authority: "operator", expected_revision: 0, command: { command: "set_enabled", enabled: false } })));
on("apply", () => runtime.apply_json(tick++));
on("select", () => runtime.select_arm_presentation_json());
on("play", () => runtime.playback_command_json(JSON.stringify({ command: "play", clip: 0 })));
on("advance", () => runtime.advance_animation_fixed_step());

show(runtime.observation_json(0, 7));
