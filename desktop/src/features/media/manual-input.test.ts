import { describe, expect, it } from "vitest";
import { manualInputOf } from "./manual-input";

const draft = {
  type: "anime" as const,
  status: "planned" as const,
  title: " Frieren ",
  year: "2023",
  total: "28",
  genres: ["Fantasy"],
};

describe("manualInputOf", () => {
  it("builds the input from a filled in form", () => {
    expect(manualInputOf(draft)).toEqual({
      media_type: "anime",
      status: "planned",
      title: "Frieren",
      cover_url: null,
      year: 2023,
      genres: ["Fantasy"],
      synopsis: "",
      progress_total: 28,
    });
  });

  it("treats blank year and total as unknown", () => {
    expect(manualInputOf({ ...draft, year: "", total: " " })).toMatchObject({
      year: null,
      progress_total: null,
    });
  });

  it("ignores the total for games", () => {
    expect(manualInputOf({ ...draft, type: "game", total: "abc" })).toMatchObject({
      progress_total: null,
    });
  });

  it("keeps the total for shows but not movies", () => {
    expect(manualInputOf({ ...draft, type: "show" })).toMatchObject({ progress_total: 28 });
    expect(manualInputOf({ ...draft, type: "movie" })).toMatchObject({ progress_total: null });
  });

  it("rejects a blank title and bad numbers", () => {
    expect(manualInputOf({ ...draft, title: "  " })).toBeNull();
    expect(manualInputOf({ ...draft, year: "20x3" })).toBeNull();
    expect(manualInputOf({ ...draft, total: "-2" })).toBeNull();
  });
});
