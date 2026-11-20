import { describe, expect, test } from "bun:test";
import { neighbours, sortPages } from "./manual-order";

const page = (id: string, order: number) => ({ id, data: { order, title: id } });

describe("sortPages", () => {
  test("sorts by order, not by file name", () => {
    const sorted = sortPages([page("b", 3), page("c", 1), page("a", 2)]);
    expect(sorted.map((p) => p.id)).toEqual(["c", "a", "b"]);
  });

  test("rejects a repeated order", () => {
    expect(() => sortPages([page("a", 1), page("b", 1)])).toThrow(/share order 1/);
  });
});

describe("neighbours", () => {
  const sorted = sortPages([page("one", 1), page("two", 2), page("three", 3)]);

  test("middle page has both", () => {
    const n = neighbours(sorted, "two");
    expect(n.prev?.id).toBe("one");
    expect(n.next?.id).toBe("three");
  });

  test("ends have one each", () => {
    expect(neighbours(sorted, "one").prev).toBeUndefined();
    expect(neighbours(sorted, "three").next).toBeUndefined();
  });

  test("unknown id has none", () => {
    expect(neighbours(sorted, "nope")).toEqual({});
  });
});
