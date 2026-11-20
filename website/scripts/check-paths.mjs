// Post-build check (ADR-0023): every internal URL in dist/ (root-relative, relative or a
// bare #fragment) must stay under the GitHub Pages base, point at a file that exists and,
// when it has a #fragment into an HTML page, at an id on that page. Catches a bare "/foo"
// link that works with `astro dev` at / but 404s on
// https://grga1349.github.io/open-camera-mouse/, a manual link like "../instalation/", and
// a stale anchor such as "manual/installation/#macos".
// Run by `bun run build`; exits non-zero with one line per problem.
// Plain ESM with node:* APIs only, so it needs no type packages and runs under Bun or Node.
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const BASE = "/open-camera-mouse/";
const ORIGIN = "https://site.invalid";
const DIST = join(dirname(fileURLToPath(import.meta.url)), "..", "dist");

// href/src/action/poster attributes, srcset candidates and CSS url(...).
const ATTR = /\s(?:href|src|action|poster)=["']([^"']+)["']/g;
const SRCSET = /\ssrcset=["']([^"']+)["']/g;
const CSS_URL = /url\(\s*["']?([^"')]+)["']?\s*\)/g;
const ID = /\sid=["']([^"']+)["']/g;
const SCHEME = /^[a-z][a-z0-9+.-]*:/i; // https:, mailto:, data:, …

/** dist-relative file a URL path serves, or null. */
function targetFile(pathname) {
  const rel = decodeURIComponent(pathname.slice(BASE.length));
  const file = join(DIST, rel);
  if (existsSync(file) && statSync(file).isFile()) return file;
  const index = join(file, "index.html");
  return existsSync(index) ? index : null;
}

const idCache = new Map();
function idsOf(file) {
  if (!idCache.has(file)) {
    const ids = new Set();
    for (const m of readFileSync(file, "utf8").matchAll(ID)) ids.add(m[1]);
    idCache.set(file, ids);
  }
  return idCache.get(file);
}

const problems = [];
let checked = 0;

function check(file, ref) {
  if (SCHEME.test(ref) || ref.startsWith("//")) return; // external
  checked++;
  // Resolve like the browser does, against the page's own URL under the base.
  const page = `${ORIGIN}${BASE}${file.split("\\").join("/")}`;
  const { pathname, hash } = new URL(ref, page);
  if (!pathname.startsWith(BASE)) {
    problems.push(`${file}: "${ref}" is outside the base ${BASE}`);
    return;
  }
  const target = targetFile(pathname);
  if (!target) {
    problems.push(`${file}: "${ref}" has no file in dist/`);
    return;
  }
  const fragment = decodeURIComponent(hash.slice(1));
  if (fragment && target.endsWith(".html") && !idsOf(target).has(fragment)) {
    problems.push(`${file}: "${ref}" points at no id="${fragment}"`);
  }
}

if (!existsSync(DIST)) {
  console.error("check-paths: dist/ not found; run `astro build` first");
  process.exit(1);
}

const files = readdirSync(DIST, { recursive: true }).filter((f) => /\.(html|css)$/.test(f));
for (const file of files) {
  const text = readFileSync(join(DIST, file), "utf8");
  const patterns = file.endsWith(".css") ? [CSS_URL] : [ATTR, CSS_URL];
  for (const re of patterns) for (const m of text.matchAll(re)) check(file, m[1]);
  if (file.endsWith(".html")) {
    for (const m of text.matchAll(SRCSET)) {
      for (const candidate of m[1].split(",")) check(file, candidate.trim().split(/\s+/)[0]);
    }
  }
}

if (problems.length > 0) {
  console.error(problems.join("\n"));
  console.error(`check-paths: ${problems.length} problem(s)`);
  process.exit(1);
}
console.log(`check-paths: ${checked} internal URL(s) and anchors under ${BASE}, all resolve`);
