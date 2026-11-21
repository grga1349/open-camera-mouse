/**
 * Order of the manual pages (sidebar, contents, previous/next). Pure, so `bun test` can
 * cover it without Astro; `src/pages/manual/` feeds it the collection entries.
 */
export interface Orderable {
  id: string;
  data: { order: number; title: string };
}

/** Pages sorted by `order`; throws on a repeated `order` so the build fails, not the sidebar. */
export function sortPages<T extends Orderable>(pages: readonly T[]): T[] {
  const sorted = [...pages].sort((a, b) => a.data.order - b.data.order);
  for (let i = 1; i < sorted.length; i++) {
    if (sorted[i].data.order === sorted[i - 1].data.order) {
      throw new Error(
        `manual: "${sorted[i - 1].id}" and "${sorted[i].id}" share order ${sorted[i].data.order}`,
      );
    }
  }
  return sorted;
}

/** The pages before and after `id` in reading order (undefined at either end). */
export function neighbours<T extends Orderable>(
  sorted: readonly T[],
  id: string,
): { prev?: T; next?: T } {
  const i = sorted.findIndex((p) => p.id === id);
  if (i < 0) return {};
  return { prev: sorted[i - 1], next: sorted[i + 1] };
}
