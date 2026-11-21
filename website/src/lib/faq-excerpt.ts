/**
 * A few questions from the manual's FAQ (`src/content/manual/faq.md`) for the home page, so
 * the answers have one source. Works on the page's rendered HTML: every `## Question` is an
 * `<h2 id>` followed by its answer. Pure, for `bun test`; used by `src/pages/index.astro`.
 */

export interface FaqItem {
  /** The heading id, e.g. "what-does-it-cost". */
  id: string;
  /** The question's inner HTML. */
  question: string;
  /** The answer's HTML: everything up to the next `<h2>`. */
  answer: string;
}

/** Split rendered FAQ HTML into its questions, in page order. */
export function splitFaq(html: string): FaqItem[] {
  const heading = /<h2 id="([^"]+)">([\s\S]*?)<\/h2>/g;
  const found = [...html.matchAll(heading)];
  return found.map((m, i) => {
    const start = m.index! + m[0].length;
    const end = i + 1 < found.length ? found[i + 1].index! : html.length;
    return { id: m[1], question: m[2].trim(), answer: html.slice(start, end).trim() };
  });
}

/** The items with these ids, in this order; throws when one is missing (a renamed question). */
export function pickFaq(items: FaqItem[], ids: string[]): FaqItem[] {
  return ids.map((id) => {
    const item = items.find((i) => i.id === id);
    if (!item) throw new Error(`FAQ has no question with id "${id}"`);
    return item;
  });
}

/**
 * Make relative links absolute paths, as seen from `pagePath` (the FAQ page, e.g.
 * "/open-camera-mouse/manual/faq/"), so they still work on another page. In-page
 * anchors (`#x`) point at the FAQ page; external links are left alone.
 */
export function rebaseLinks(html: string, pagePath: string): string {
  const origin = "https://site.invalid";
  return html.replace(/href="([^"]*)"/g, (whole, href: string) => {
    if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("/")) return whole;
    const u = new URL(href, origin + pagePath);
    return `href="${u.pathname}${u.hash}"`;
  });
}
