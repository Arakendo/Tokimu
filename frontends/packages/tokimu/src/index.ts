// The `tokimu` package is the authoring anchor described in the TTSDD.
//
// Tokimu recognizes calls that originate from this package (and the wider
// `@tokimu/*` family) and lowers or runs them. It deliberately does not try to
// understand arbitrary TypeScript. Re-exporting the stable authoring surface
// here is what lets authors write `import { rule, query } from "tokimu"`.
export * from "@tokimu/rules";
