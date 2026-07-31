import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceUrl = new URL(
  "../../docs/Libraries/cgm-corpus-testing.md",
  import.meta.url,
);
const pageUrl = new URL("../docs/formats/cgm.md", import.meta.url);

test("the public CGM page matches the authoritative bounded corpus record", async () => {
  const [source, page] = await Promise.all([
    readFile(sourceUrl, "utf8"),
    readFile(pageUrl, "utf8"),
  ]);

  const evidenceDate = source.match(/^As of (\d{4}-\d{2}-\d{2}),/m);
  assert.ok(evidenceDate, "authoritative CGM evidence date is present");
  assert.match(source, /all 26 selected fixtures are now registered/i);
  assert.match(source, /Thirteen cases reach a successful source-to-vector\s+boundary/i);
  assert.match(source, /No CGM importer\s+or first-party capability has been\s+admitted yet/i);
  assert.match(
    source,
    /twelve admitted primitive and source-state passes plus one\s+expected polygon-set topology boundary/i,
  );

  assert.match(page, /26 \/ 26 verified/);
  assert.match(page, /Cases registered in the shared runner \| 26 \/ 26/);
  assert.match(page, /Successful source-to-vector cases \| 13/);
  assert.match(page, /Expected vector boundary \| 1/);
  assert.match(page, /Source-only cases \| 12/);
  assert.match(page, /Admitted production importer \| 0/);
  assert.match(page, new RegExp(`datetime="${evidenceDate[1]}"`));
  assert.match(page, /<strong>Previewable<\/strong>/);
  assert.match(page, /diagnostic outline evidence/);
  assert.match(page, /selection-v1\.toml/);
  assert.match(page, /cgm_cases\.rs/);
  assert.match(page, /cgm_artifacts\.rs/);
  assert.match(page, /does not claim:\s*\n\n- complete CGM, WebCGM, CALS, or ISO conformance/);
});
