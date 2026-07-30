import assert from "node:assert/strict";
import { access, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");
const siteRoot = path.join(repositoryRoot, "target", "website");
const canonicalOrigin = "https://tokimuengine.org";

async function collectFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const candidate = path.join(directory, entry.name);
    return entry.isDirectory() ? collectFiles(candidate) : [candidate];
  }));
  return nested.flat();
}

function htmlAttribute(source, relation, attribute) {
  const tags = source.match(/<(?:link|meta)\b[^>]*>/gi) ?? [];
  const tag = tags.find((candidate) => relation.test(candidate));
  return tag?.match(new RegExp(`${attribute}=["']([^"']+)["']`, "i"))?.[1];
}

async function resolvesInsideSite(href, pageFile) {
  if (
    href.startsWith("#")
    || href.startsWith("mailto:")
    || href.startsWith("tel:")
    || href.startsWith("data:")
    || href.startsWith("javascript:")
  ) {
    return true;
  }

  const pageRoute = `/${path.relative(siteRoot, pageFile).replaceAll("\\", "/")}`;
  const pageUrl = new URL(pageRoute, `${canonicalOrigin}/`);
  const targetUrl = new URL(href, pageUrl);
  if (targetUrl.origin !== canonicalOrigin) {
    return true;
  }

  const pathname = decodeURIComponent(targetUrl.pathname);
  const relative = pathname.replace(/^\/+/, "");
  const candidates = pathname.endsWith("/")
    ? [path.join(siteRoot, relative, "index.html")]
    : [
        path.join(siteRoot, relative),
        path.join(siteRoot, relative, "index.html"),
        path.join(siteRoot, `${relative}.html`),
      ];

  for (const candidate of candidates) {
    try {
      await access(candidate);
      return true;
    } catch {
      // Try the next MkDocs output shape.
    }
  }
  return false;
}

test("generated site has GitHub Pages identity files", async () => {
  assert.equal((await readFile(path.join(siteRoot, "CNAME"), "utf8")).trim(), "tokimuengine.org");
  await access(path.join(siteRoot, ".nojekyll"));
});

test("every generated page has canonical metadata and a description", async () => {
  const files = await collectFiles(siteRoot);
  const pages = files.filter((file) => file.endsWith(".html"));
  assert.ok(pages.length >= 10, "expected the complete documentation site");

  for (const page of pages) {
    const source = await readFile(page, "utf8");
    const canonical = htmlAttribute(source, /\brel=["']canonical["']/i, "href");
    const description = htmlAttribute(source, /\bname=["']description["']/i, "content");
    assert.ok(canonical?.startsWith(`${canonicalOrigin}/`), `${page} has a canonical URL`);
    assert.ok(description?.trim(), `${page} has a non-empty description`);
  }
});

test("every generated page preserves the static accessibility shell", async () => {
  const files = await collectFiles(siteRoot);
  const pages = files.filter((file) => file.endsWith(".html"));

  for (const page of pages) {
    const source = await readFile(page, "utf8");
    const relative = path.relative(siteRoot, page);
    const ids = [...source.matchAll(/\bid=["']([^"']+)["']/gi)].map((match) => match[1]);
    const duplicateIds = ids.filter((id, index) => ids.indexOf(id) !== index);

    assert.match(source, /<html\b[^>]*\blang=["']en["']/i, `${relative} declares English`);
    assert.match(
      source,
      /<meta\b[^>]*\bname=["']viewport["'][^>]*>/i,
      `${relative} has responsive viewport metadata`,
    );
    assert.match(
      source,
      /<a\b[^>]*class=["'][^"']*\bskip-link\b[^"']*["'][^>]*href=["']#content["'][^>]*>/i,
      `${relative} provides a skip link`,
    );
    assert.equal(
      (source.match(/<main\b/gi) ?? []).length,
      1,
      `${relative} has exactly one main landmark`,
    );
    assert.match(
      source,
      /<nav\b[^>]*aria-label=["']Primary navigation["']/i,
      `${relative} labels primary navigation`,
    );
    assert.equal(
      (source.match(/<h1\b/gi) ?? []).length,
      1,
      `${relative} has exactly one page heading`,
    );
    assert.deepEqual(duplicateIds, [], `${relative} has no duplicate element IDs`);
  }
});

test("generated internal links and linked assets resolve", async () => {
  const files = await collectFiles(siteRoot);
  const pages = files.filter((file) => file.endsWith(".html"));
  const failures = [];

  for (const page of pages) {
    const source = await readFile(page, "utf8");
    const hrefs = [...source.matchAll(/\bhref=["']([^"']+)["']/gi)].map((match) => match[1]);
    for (const href of hrefs) {
      if (!(await resolvesInsideSite(href, page))) {
        failures.push(`${path.relative(siteRoot, page)} -> ${href}`);
      }
    }
  }

  assert.deepEqual(failures, []);
});

test("homepage remains useful without JavaScript", async () => {
  const source = await readFile(path.join(siteRoot, "index.html"), "utf8");
  assert.match(source, /Build interactive systems around/);
  assert.match(source, /Asset observation workbench/);
  assert.match(source, /Rust owns parsing and vector lowering/);
  assert.doesNotMatch(source, /WASM planned/);
});
