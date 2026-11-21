/**
 * Prefix a site-internal path with the deploy base (`/open-camera-mouse/` on GitHub Pages).
 *
 * `url("/")` → `/open-camera-mouse/`, `url("favicon.png")` → `/open-camera-mouse/favicon.png`.
 * Page paths must end in `/` (`trailingSlash: "always"`); file paths keep their extension.
 * External URLs (`https:`, `mailto:`) and in-page anchors (`#id`) are returned unchanged.
 */
export function url(path: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(path) || path.startsWith("#")) return path;
  const base = import.meta.env.BASE_URL.replace(/\/+$/, "");
  return `${base}/${path.replace(/^\/+/, "")}`;
}

/** The project repository; the only external destination the scaffold links to. */
export const REPO_URL = "https://github.com/grga1349/open-camera-mouse";
