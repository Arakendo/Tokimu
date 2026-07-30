import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceUrl = new URL(
  "../../docs/Libraries/w3c-svg-corpus-testing.md",
  import.meta.url,
);
const pageUrl = new URL("../docs/formats/svg.md", import.meta.url);

function readEvidence(source) {
  const coverage = source.match(
    /represented by the manifest \| (\d+) \/ (\d+) \| \*\*(\d+\.\d+)%\*\*/,
  );
  assert.ok(coverage, "authoritative SVG coverage row is present");

  const manifestEntries = source.match(
    /\| Selection manifest entries \| (\d+) \|/,
  );
  assert.ok(manifestEntries, "authoritative manifest entry count is present");

  const runnerCases = source.match(
    /\| Registered SVG runner cases \| (\d+) \|/,
  );
  assert.ok(runnerCases, "authoritative SVG runner count is present");

  const structuralGoldens = source.match(
    /\| Registered structural goldens, all producers \| (\d+) \/ (\d+) \|/,
  );
  assert.ok(structuralGoldens, "authoritative structural golden count is present");

  const evidenceDate = source.match(/Active and structurally validated as of (\d{4}-\d{2}-\d{2})/);
  assert.ok(evidenceDate, "authoritative SVG evidence date is present");

  return {
    numerator: coverage[1],
    denominator: coverage[2],
    percentage: coverage[3],
    manifestEntries: manifestEntries[1],
    runnerCases: runnerCases[1],
    reviewedGoldens: structuralGoldens[1],
    totalGoldens: structuralGoldens[2],
    date: evidenceDate[1],
  };
}

test("the public SVG page matches the authoritative corpus record", async () => {
  const [source, page] = await Promise.all([
    readFile(sourceUrl, "utf8"),
    readFile(pageUrl, "utf8"),
  ]);
  const evidence = readEvidence(source);

  assert.match(
    page,
    new RegExp(
      `${evidence.numerator} / ${evidence.denominator} · ${evidence.percentage}%`,
    ),
  );
  assert.match(page, new RegExp(`datetime="${evidence.date}"`));
  assert.match(page, new RegExp(`Selection manifest entries \\| ${evidence.manifestEntries}`));
  assert.match(page, new RegExp(`Registered SVG runner cases \\| ${evidence.runnerCases}`));
  assert.match(
    page,
    new RegExp(
      `Reviewed structural goldens \\| ${evidence.reviewedGoldens} / ${evidence.totalGoldens}`,
    ),
  );
  assert.match(page, /Structural artifacts are authoritative|artifacts are authoritative/);
  assert.match(page, /Browser visual/);
  assert.match(page, /does not mean that Tokimu is 7\.62% compliant/);
  assert.match(page, /selection-v1\.toml/);
  assert.match(page, /w3c_svg_cases\.rs/);
  assert.match(page, /golden_workflow\.rs/);
  assert.match(page, /provenance\.json/);
  assert.match(page, /Native-window\s+screenshots remain separately labeled manual evidence/);
});
