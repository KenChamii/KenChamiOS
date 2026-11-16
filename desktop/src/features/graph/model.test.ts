import { describe, expect, it } from "vitest";
import type { GraphData } from "@/lib/api/vault";
import { degrees, neighborsOf, obsidianUrl, visibleGraph, withinSteps } from "./model";

const data: GraphData = {
  nodes: [
    { id: "a", kind: "note", label: "Alpha", path: "Alpha.md" },
    { id: "b", kind: "note", label: "Beta", path: "Beta.md" },
    { id: "c", kind: "project", label: "Gamma", path: null },
    { id: "d", kind: "tag", label: "delta", path: null },
    { id: "e", kind: "note", label: "Epsilon", path: "Epsilon.md" },
  ],
  edges: [
    { source: "a", target: "b", kind: "wikilink" },
    { source: "b", target: "c", kind: "wikilink" },
    { source: "c", target: "d", kind: "tag" },
  ],
};

const all = new Set(["note", "project", "daily", "tag"] as const);
const options = { kinds: all, query: "", focusId: null, depth: 2 };

describe("withinSteps", () => {
  it("walks the given number of steps", () => {
    const neighbors = neighborsOf(data.edges);
    expect([...withinSteps(neighbors, "a", 0)]).toEqual(["a"]);
    expect([...withinSteps(neighbors, "a", 1)].sort()).toEqual(["a", "b"]);
    expect([...withinSteps(neighbors, "a", 2)].sort()).toEqual(["a", "b", "c"]);
  });
});

describe("visibleGraph", () => {
  it("hides kinds that are switched off, and the edges that lose an end", () => {
    const kinds = new Set(["note", "project"] as const);
    const result = visibleGraph(data, { ...options, kinds });
    expect(result.nodes.map((n) => n.id)).toEqual(["a", "b", "c", "e"]);
    expect(result.edges).toHaveLength(2);
  });

  it("shows only the local graph of the focused node", () => {
    const result = visibleGraph(data, { ...options, focusId: "a", depth: 2 });
    expect(result.nodes.map((n) => n.id)).toEqual(["a", "b", "c"]);
  });

  it("searches labels without case", () => {
    expect(visibleGraph(data, { ...options, query: "EPS" }).nodes.map((n) => n.id)).toEqual(["e"]);
  });
});

describe("degrees", () => {
  it("counts links at both ends", () => {
    const counts = degrees(data.edges);
    expect(counts.get("b")).toBe(2);
    expect(counts.get("e")).toBeUndefined();
  });
});

describe("obsidianUrl", () => {
  it("names the vault by its folder and drops the extension", () => {
    expect(obsidianUrl("C:\\Obsidian\\KenChami Obsidian\\", "Ideas/My note.md")).toBe(
      "obsidian://open?vault=KenChami%20Obsidian&file=Ideas%2FMy%20note",
    );
  });
});
