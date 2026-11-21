// @ts-check
import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

// GitHub Pages project site: everything is served under /open-camera-mouse/ (ADR-0023).
// Build internal URLs with `url()` from src/lib/url.ts, never as bare "/…" strings.
export default defineConfig({
  site: "https://grga1349.github.io",
  base: "/open-camera-mouse/",
  trailingSlash: "always",
  output: "static",
  build: { format: "directory" },
  // Manual code blocks stay monochrome: no Shiki colour theme (WEBSITE_DESIGN.md "Brand").
  markdown: { syntaxHighlight: false },
  vite: { plugins: [tailwindcss()] },
});
