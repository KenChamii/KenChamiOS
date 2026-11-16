import { describe, expect, it } from "vitest";
import { continueQueue, nextUnit, queueTypes, upNextQueue } from "./queues";
import { testEntry } from "./test-entry";

describe("queues", () => {
  const entries = [
    testEntry({ id: "old", status: "in_progress", updated_at: 1 }),
    testEntry({ id: "new", status: "in_progress", updated_at: 9 }),
    testEntry({ id: "plan-b", status: "planned", created_at: 5 }),
    testEntry({ id: "plan-a", status: "planned", created_at: 2 }),
    testEntry({ id: "game", media_type: "game", status: "in_progress" }),
    testEntry({ id: "done", status: "completed" }),
  ];

  it("continues the title touched last first, per type", () => {
    expect(continueQueue(entries, "anime").map((e) => e.id)).toEqual(["new", "old"]);
    expect(continueQueue(entries, "game").map((e) => e.id)).toEqual(["game"]);
  });

  it("queues planned titles by when they were added", () => {
    expect(upNextQueue(entries, "anime").map((e) => e.id)).toEqual(["plan-a", "plan-b"]);
  });

  it("only offers types that have something queued", () => {
    expect(queueTypes(entries, ["manga", "anime", "game", "movie"])).toEqual(["anime", "game"]);
  });
});

describe("nextUnit", () => {
  it("names the next episode or chapter", () => {
    expect(nextUnit(testEntry({ progress: 4 }))).toBe("Episode 5");
    expect(nextUnit(testEntry({ media_type: "manga", progress: 0 }))).toBe("Chapter 1");
  });

  it("is empty at the end, and for types without units", () => {
    expect(nextUnit(testEntry({ progress: 12, progress_total: 12 }))).toBeNull();
    expect(nextUnit(testEntry({ media_type: "game" }))).toBeNull();
  });
});
