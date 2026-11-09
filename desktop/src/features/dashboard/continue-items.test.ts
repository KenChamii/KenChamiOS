import { describe, expect, it } from "vitest";
import { testEntry } from "@/features/media/test-entry";
import { continueItems, nextLabel } from "./continue-items";

describe("continueItems", () => {
  const entries = [
    testEntry({ id: "old", status: "in_progress", updated_at: 1 }),
    testEntry({ id: "new", status: "in_progress", media_type: "manga", updated_at: 9 }),
    testEntry({ id: "planned", status: "planned", updated_at: 20 }),
    testEntry({ id: "film", status: "in_progress", media_type: "movie", updated_at: 30 }),
    testEntry({ id: "album", status: "in_progress", media_type: "album", updated_at: 40 }),
    testEntry({ id: "game", status: "in_progress", media_type: "game", updated_at: 5 }),
  ];

  it("keeps in progress titles that can be continued, newest first", () => {
    expect(continueItems(entries).map((e) => e.id)).toEqual(["new", "game", "old"]);
  });

  it("shows at most six", () => {
    const many = Array.from({ length: 9 }, (_, index) =>
      testEntry({ id: `e${index}`, status: "in_progress", updated_at: index }),
    );

    expect(continueItems(many)).toHaveLength(6);
  });
});

describe("nextLabel", () => {
  it("names the next episode or chapter", () => {
    expect(nextLabel({ media_type: "anime", progress: 12 })).toBe("Episode 13");
    expect(nextLabel({ media_type: "show", progress: 0 })).toBe("Episode 1");
    expect(nextLabel({ media_type: "manga", progress: 140 })).toBe("Chapter 141");
  });

  it("shows hours for games", () => {
    expect(nextLabel({ media_type: "game", progress: 0 })).toBe("Just started");
    expect(nextLabel({ media_type: "game", progress: 30 })).toBe("30 h played");
  });
});
