import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceUrl = new URL(
  "../../docs/Libraries/w3c-svg-corpus-testing.md",
  import.meta.url,
);
const pageUrl = new URL("../docs/formats/svg.md", import.meta.url);

function readCoverage(source) {
  const coverage = source.match(
    /represented by the manifest \| (\d+) \/ (\d+) \| \*\*(\d+\.\d+)%\*\*/,
  );
  assert.ok(coverage, "authoritative SVG coverage row is present");

  const evidenceDate = source.match(/Active and structurally validated as of (\d{4}-\d{2}-\d{2})/);
  assert.ok(evidenceDate, "authoritative SVG evidence date is present");

  return {
    numerator: coverage[1],
    denominator: coverage[2],
    percentage: coverage[3],
    date: evidenceDate[1],
  };
}

test("the public SVG page matches the authoritative corpus record", async () => {
  const [source, page] = await Promise.all([
    readFile(sourceUrl, "utf8"),
    readFile(pageUrl, "utf8"),
  ]);
  const evidence = readCoverage(source);

  assert.match(
    page,
    new RegExp(
      `${evidence.numerator} / ${evidence.denominator} · ${evidence.percentage}%`,
    ),
  );
  assert.match(page, new RegExp(`datetime="${evidence.date}"`));
  assert.match(page, /Structural artifacts are authoritative|artifacts are authoritative/);
  assert.match(page, /Browser visual/);
  assert.match(page, /does not mean that Tokimu is 7\.62% compliant/);
});
