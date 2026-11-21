// Content collections (ADR-0023, WEBSITE.md "Manual"). Astro's built-in glob loader, no
// docs framework: each src/content/manual/<slug>.md becomes /manual/<slug>/.
import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "astro/zod";

const manual = defineCollection({
  loader: glob({ pattern: "*.md", base: "./src/content/manual" }),
  schema: z.object({
    title: z.string(),
    /** One sentence: the page's meta description and its line in the contents. */
    description: z.string(),
    /** Position in the sidebar and in previous/next; unique, ascending. */
    order: z.number().int(),
  }),
});

export const collections = { manual };
