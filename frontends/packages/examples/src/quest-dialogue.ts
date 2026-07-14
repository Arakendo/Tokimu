// Runtime example.
//
// This rule declares `execution: "runtime"`. It is orchestration logic that does
// not need engine-owned determinism, so it stays in the TypeScript runtime host
// and is reported as runtime-only rather than lowered — on purpose.
import { rule } from "tokimu";

export const questDialogue = rule("quest-dialogue", {
  execution: "runtime",
  inputs: ["QuestState"],
  outputs: ["DialogueUI"],
  signals: ["dialogue-opened"],
  run(ctx) {
    // Flexible orchestration: open a dialogue and announce it. The runtime host
    // supplies the real effect behind a narrow API; nothing here is lowered.
    ctx.emit("dialogue-opened");
  },
});
