import { describe, expect, it } from "vitest";
import type { LibraryEntry } from "@/lib/api/media";
import { fromDateInput } from "@/lib/dates";
import { draftOf, inputOf } from "./entry-input";

const entry: LibraryEntry = {
  id: "e",
  created_at: 0,
  updated_at: 0,
  media_item_id: "i",
  media_type: "game",
  title: "Hollow Knight",
  cover_url: null,
  cover_cache_path: null,
  cover_path: null,
  year: 2017,
  genres: [],
  synopsis: "",
  source: "manual",
  external_id: null,
  status: "in_progress",
  progress: 12,
  progress_total: null,
  rating: 9,
  notes: "great",
  tags: ["metroidvania", "fav"],
  started_at: fromDateInput("2026-01-02"),
  finished_at: null,
  platform: "PC",
};

describe("draftOf and inputOf", () => {
  it("round trips an entry", () => {
    const input = inputOf(draftOf(entry));

    expect(input).toEqual({
      status: "in_progress",
      progress_total: null,
      rating: 9,
      notes: "great",
      tags: ["metroidvania", "fav"],
      started_at: fromDateInput("2026-01-02"),
      finished_at: null,
      platform: "PC",
    });
  });

  it("treats blank fields as no value", () => {
    const draft = { ...draftOf(entry), rating: " ", total: "", platform: "  ", tags: " , " };

    expect(inputOf(draft)).toMatchObject({
      rating: null,
      progress_total: null,
      platform: null,
      tags: [],
    });
  });

  it("rejects ratings outside 1 to 10 and bad totals", () => {
    for (const rating of ["0", "11", "7.5", "abc"]) {
      expect(inputOf({ ...draftOf(entry), rating })).toBeNull();
    }
    for (const total of ["-1", "2.5", "x"]) {
      expect(inputOf({ ...draftOf(entry), total })).toBeNull();
    }
    expect(inputOf({ ...draftOf(entry), rating: "10", total: "0" })).not.toBeNull();
  });
});
