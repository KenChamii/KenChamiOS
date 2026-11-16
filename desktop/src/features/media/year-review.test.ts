import { describe, expect, it } from "vitest";
import { testEntry } from "./test-entry";
import { reviewYears, yearReview } from "./year-review";

const at = (year: number, month: number) => new Date(year, month - 1, 15).getTime();

const library = [
  testEntry({
    id: "a",
    title: "A",
    media_type: "anime",
    status: "completed",
    finished_at: at(2026, 1),
    genres: ["Action", "Drama"],
    rating: 9,
    progress: 12,
  }),
  testEntry({
    id: "b",
    title: "B",
    media_type: "game",
    status: "completed",
    finished_at: at(2026, 3),
    genres: ["Action"],
    rating: 10,
    progress: 40,
  }),
  testEntry({
    id: "c",
    title: "C",
    media_type: "manga",
    status: "completed",
    finished_at: at(2026, 3),
    progress: 30,
  }),
  testEntry({ id: "d", status: "completed", finished_at: at(2025, 6), progress: 5 }),
  testEntry({ id: "e", status: "in_progress", finished_at: null }),
];

describe("reviewYears", () => {
  it("lists years with finished titles, newest first", () => {
    expect(reviewYears(library)).toEqual([2026, 2025]);
  });
});

describe("yearReview", () => {
  it("counts finished titles per month", () => {
    const review = yearReview(library, 2026);
    expect(review.finished).toBe(3);
    expect(review.byMonth).toEqual([1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  });

  it("adds up game hours, episodes and chapters of that year only", () => {
    const review = yearReview(library, 2026);
    expect(review).toMatchObject({ gameHours: 40, episodes: 12, chapters: 30 });
  });

  it("ranks genres and ratings", () => {
    const review = yearReview(library, 2026);
    expect(review.topGenres[0]).toEqual({ genre: "Action", count: 2 });
    expect(review.highestRated.map((entry) => entry.id)).toEqual(["b", "a"]);
  });

  it("is empty for a year with nothing finished", () => {
    expect(yearReview(library, 2020)).toMatchObject({ finished: 0, topGenres: [] });
  });
});
