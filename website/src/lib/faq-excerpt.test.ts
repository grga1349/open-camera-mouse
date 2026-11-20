import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { pickFaq, rebaseLinks, splitFaq } from "./faq-excerpt";
import { HOME_FAQ } from "./home-faq";

const html = `<h2 id="a">Question A?</h2>
<p>Answer A, <a href="../settings/#x">settings</a>.</p>
<h2 id="b">Question B?</h2>
<p>Answer B.</p>
<p>More B, <a href="https://example.org/">site</a>.</p>`;

describe("splitFaq", () => {
  test("one item per h2 with the answer up to the next h2", () => {
    const items = splitFaq(html);
    expect(items.map((i) => i.id)).toEqual(["a", "b"]);
    expect(items[0].question).toBe("Question A?");
    expect(items[0].answer).toBe(`<p>Answer A, <a href="../settings/#x">settings</a>.</p>`);
    expect(items[1].answer).toContain("More B");
  });
});

describe("pickFaq", () => {
  test("keeps the requested order", () => {
    expect(pickFaq(splitFaq(html), ["b", "a"]).map((i) => i.id)).toEqual(["b", "a"]);
  });

  test("fails on a missing question", () => {
    expect(() => pickFaq(splitFaq(html), ["gone"])).toThrow(/gone/);
  });
});

describe("rebaseLinks", () => {
  const page = "/open-camera-mouse/manual/faq/";

  test("relative links resolve from the FAQ page", () => {
    expect(rebaseLinks(`<a href="../settings/#x">`, page)).toBe(`<a href="/open-camera-mouse/manual/settings/#x">`);
    expect(rebaseLinks(`<a href="#y">`, page)).toBe(`<a href="/open-camera-mouse/manual/faq/#y">`);
  });

  test("external and absolute links stay", () => {
    expect(rebaseLinks(`<a href="https://example.org/">`, page)).toBe(`<a href="https://example.org/">`);
    expect(rebaseLinks(`<a href="/x/">`, page)).toBe(`<a href="/x/">`);
  });
});

describe("home page FAQ", () => {
  // Heading ids as Astro generates them (lower case, punctuation dropped, spaces → "-").
  const slug = (s: string) =>
    s
      .toLowerCase()
      .replace(/[^a-z0-9 -]/g, "")
      .trim()
      .replace(/ /g, "-");
  const ids = readFileSync(new URL("../content/manual/faq.md", import.meta.url), "utf8")
    .split("\n")
    .filter((l) => l.startsWith("## "))
    .map((l) => slug(l.slice(3)));

  test("every chosen question exists in faq.md", () => {
    for (const id of HOME_FAQ) expect(ids).toContain(id);
  });
});
