import { describe, expect, it } from "vitest";
import type { Prompt } from "@/lib/api/prompts";
import { filterPrompts, type PromptFilter } from "./filter";

function prompt(overrides: Partial<Prompt>): Prompt {
  return {
    id: "p",
    created_at: 0,
    project_id: null,
    title: "Untitled",
    body_md: "",
    tags: [],
    favorite: false,
    use_count: 0,
    last_used_at: null,
    ...overrides,
  };
}

const ALL: PromptFilter = { search: "", scope: "all", favoritesOnly: false };

const prompts = [
  prompt({ id: "review", title: "Code review", tags: ["quality"], favorite: true }),
  prompt({
    id: "scoped",
    title: "Release notes",
    project_id: "proj-1",
    body_md: "Summarise commits",
  }),
  prompt({ id: "other", title: "Debug helper", project_id: "proj-2" }),
];

function ids(filter: Partial<PromptFilter>) {
  return filterPrompts(prompts, { ...ALL, ...filter }).map((p) => p.id);
}

describe("filterPrompts", () => {
  it("returns everything by default, keeping the order", () => {
    expect(ids({})).toEqual(["review", "scoped", "other"]);
  });

  it("narrows by scope", () => {
    expect(ids({ scope: "global" })).toEqual(["review"]);
    expect(ids({ scope: "proj-1" })).toEqual(["scoped"]);
  });

  it("keeps only favorites when asked", () => {
    expect(ids({ favoritesOnly: true })).toEqual(["review"]);
  });

  it("searches title, body and tags without case", () => {
    expect(ids({ search: "REVIEW" })).toEqual(["review"]);
    expect(ids({ search: "commits" })).toEqual(["scoped"]);
    expect(ids({ search: "qual" })).toEqual(["review"]);
    expect(ids({ search: "nothing" })).toEqual([]);
  });

  it("combines the filters", () => {
    expect(ids({ scope: "proj-1", search: "release", favoritesOnly: true })).toEqual([]);
    expect(ids({ scope: "proj-1", search: "release" })).toEqual(["scoped"]);
  });
});
