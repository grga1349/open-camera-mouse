// Post-build check (ADR-0023): every root-relative URL in dist/ must stay under the GitHub
// Pages base and point at a file that exists. Catches a bare "/foo" link that works with
// `astro dev` at / but 404s on https://grga1349.github.io/open-camera-mouse/.
// Run by `bun run build`; exits non-zero with one line per problem.
// Plain ESM with node:* APIs only, so it needs no type packages and runs under Bun or Node.
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const BASE = "/open-camera-mouse/";
const DIST = join(dirname(fileURLToPath(import.meta.url)), "..", "dist");

// href/src/action/poster attributes, srcset candidates and CSS url(...).
const ATTR = /\s(?:href|src|action|poster)=["']([^"']+)["']/g;
const SRCSET = /\ssrcset=["']([^"']+)["']/g;
const CSS_URL = /url\(\s*["']?([^"')]+)["']?\s*\)/g;

function targetExists(path) {
  const rel = decodeURIComponent(path.slice(BASE.length).split(/[?#]/)[0]);
  const file = join(DIST, rel);
  if (existsSync(file) && statSync(file).isFile()) return true;
  return existsSync(join(file, "index.html"));
}

const problems = [];
let checked = 0;

function check(file, ref) {
  if (!ref.startsWith("/") || ref.startsWith("//")) return; // relative, external or anchor
  checked++;
  if (!ref.startsWith(BASE)) problems.push(`${file}: "${ref}" is outside the base ${BASE}`);
  else if (!targetExists(ref)) problems.push(`${file}: "${ref}" has no file in dist/`);
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
console.log(`check-paths: ${checked} internal URL(s) under ${BASE}, all resolve`);
