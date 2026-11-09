import { describe, expect, it } from "vitest";
import { backlog, NO_PICKER_FILTER, pickNext, remaining } from "./picker";
import { testEntry } from "./test-entry";

const entries = [
  testEntry({ id: "a", genres: ["Fantasy"], tags: ["cozy"], progress_total: 12 }),
  testEntry({ id: "b", media_type: "game", genres: ["Action"], tags: ["hard"] }),
  testEntry({ id: "c", media_type: "manga", progress: 190, progress_total: 200 }),
  testEntry({ id: "d", status: "in_progress" }),
  testEntry({ id: "e", status: "completed" }),
];

function ids(filter: Partial<typeof NO_PICKER_FILTER>) {
  return backlog(entries, { ...NO_PICKER_FILTER, ...filter }).map((e) => e.id);
}

describe("backlog", () => {
  it("only offers planned entries", () => {
    expect(ids({})).toEqual(["a", "b", "c"]);
  });

  it("narrows by type, genre and mood tag", () => {
    expect(ids({ type: "game" })).toEqual(["b"]);
    expect(ids({ genre: "Fantasy" })).toEqual(["a"]);
    expect(ids({ tag: "hard" })).toEqual(["b"]);
  });

  it("limits by what is left and drops entries of unknown length", () => {
    expect(ids({ maxLeft: 10 })).toEqual(["c"]);
    expect(ids({ maxLeft: 12 })).toEqual(["a", "c"]);
    expect(ids({ maxLeft: 0 })).toEqual([]);
  });
});

describe("remaining", () => {
  it("is the total minus progress, or null without a total", () => {
    expect(remaining({ progress: 3, progress_total: 12 })).toBe(9);
    expect(remaining({ progress: 3, progress_total: null })).toBeNull();
    expect(remaining({ progress: 15, progress_total: 12 })).toBe(0);
  });
});

describe("pickNext", () => {
  it("picks by the random value", () => {
    expect(pickNext(entries.slice(0, 3), null, () => 0)?.id).toBe("a");
    expect(pickNext(entries.slice(0, 3), null, () => 0.99)?.id).toBe("c");
  });

  it("avoids the current pick while there is another choice", () => {
    expect(pickNext(entries.slice(0, 2), "a", () => 0)?.id).toBe("b");
    expect(pickNext(entries.slice(0, 1), "a", () => 0)?.id).toBe("a");
  });

  it("returns null without candidates", () => {
    expect(pickNext([], null)).toBeNull();
  });
});
