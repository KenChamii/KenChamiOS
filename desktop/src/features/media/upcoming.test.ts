import { describe, expect, it } from "vitest";
import type { UpcomingRelease } from "@/lib/api/media";
import { testEntry } from "./test-entry";
import { groupUpcoming } from "./upcoming";

const NOW = new Date(2026, 10, 16, 12, 0).getTime();
const inDays = (days: number, hour = 9) => new Date(2026, 10, 16 + days, hour, 0).getTime();

const release = (id: string, air_at_ms: number): UpcomingRelease => ({
  entry: testEntry({ id }),
  episode: 1,
  air_at_ms,
});

describe("groupUpcoming", () => {
  it("groups by how soon, keeping the order given", () => {
    const groups = groupUpcoming(
      [
        release("a", inDays(0, 20)),
        release("b", inDays(1)),
        release("c", inDays(3)),
        release("d", inDays(9)),
        release("e", inDays(3, 18)),
      ],
      NOW,
    );
    expect(groups.map((g) => [g.label, g.releases.map((r) => r.entry.id)])).toEqual([
      ["Today", ["a"]],
      ["Tomorrow", ["b"]],
      ["This week", ["c", "e"]],
      ["Later", ["d"]],
    ]);
  });

  it("leaves out empty groups, and treats an earlier time today as today", () => {
    const groups = groupUpcoming([release("a", inDays(0, 6))], NOW);
    expect(groups.map((g) => g.label)).toEqual(["Today"]);
  });
});
