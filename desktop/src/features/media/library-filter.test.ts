import { describe, expect, it } from "vitest";
import type { LibraryEntry } from "@/lib/api/media";
import { distinctValues, filterEntries, NO_FILTER, type LibraryFilter } from "./library-filter";

function entry(overrides: Partial<LibraryEntry>): LibraryEntry {
  return {
    id: "e",
    created_at: 0,
    updated_at: 0,
    media_item_id: "i",
    media_type: "anime",
    title: "Untitled",
    cover_url: null,
    cover_cache_path: null,
    cover_path: null,
    year: null,
    genres: [],
    synopsis: "",
    source: "manual",
    external_id: null,
    status: "planned",
    progress: 0,
    progress_total: null,
    rating: null,
    notes: "",
    tags: [],
    started_at: null,
    finished_at: null,
    platform: null,
    ...overrides,
  };
}

const entries = [
  entry({
    id: "frieren",
    title: "Frieren",
    status: "in_progress",
    genres: ["Fantasy"],
    rating: 9,
    updated_at: 30,
    created_at: 1,
  }),
  entry({
    id: "berserk",
    title: "Berserk",
    media_type: "manga",
    genres: ["Fantasy", "Horror"],
    tags: ["dark"],
    rating: 10,
    updated_at: 10,
    created_at: 3,
  }),
  entry({
    id: "hollow",
    title: "Hollow Knight",
    media_type: "game",
    status: "completed",
    updated_at: 20,
    created_at: 2,
  }),
];

function ids(filter: Partial<LibraryFilter>) {
  return filterEntries(entries, { ...NO_FILTER, ...filter }).map((e) => e.id);
}

describe("filterEntries", () => {
  it("returns everything, most recently changed first", () => {
    expect(ids({})).toEqual(["frieren", "hollow", "berserk"]);
  });

  it("narrows by type, status, genre and tag", () => {
    expect(ids({ type: "manga" })).toEqual(["berserk"]);
    expect(ids({ status: "completed" })).toEqual(["hollow"]);
    expect(ids({ genre: "Fantasy" })).toEqual(["frieren", "berserk"]);
    expect(ids({ tag: "dark" })).toEqual(["berserk"]);
  });

  it("searches titles without case", () => {
    expect(ids({ search: " KNIGHT " })).toEqual(["hollow"]);
    expect(ids({ search: "nothing" })).toEqual([]);
  });

  it("combines the filters", () => {
    expect(ids({ genre: "Fantasy", type: "anime" })).toEqual(["frieren"]);
    expect(ids({ genre: "Horror", status: "in_progress" })).toEqual([]);
  });

  it("sorts by title, date added and rating with unrated last", () => {
    expect(ids({ sort: "title" })).toEqual(["berserk", "frieren", "hollow"]);
    expect(ids({ sort: "added" })).toEqual(["berserk", "hollow", "frieren"]);
    expect(ids({ sort: "rating" })).toEqual(["berserk", "frieren", "hollow"]);
  });

  it("does not reorder the list it was given", () => {
    const before = entries.map((e) => e.id);
    ids({ sort: "title" });
    expect(entries.map((e) => e.id)).toEqual(before);
  });
});

describe("distinctValues", () => {
  it("lists each genre or tag once, sorted", () => {
    expect(distinctValues(entries, "genres")).toEqual(["Fantasy", "Horror"]);
    expect(distinctValues(entries, "tags")).toEqual(["dark"]);
  });
});
