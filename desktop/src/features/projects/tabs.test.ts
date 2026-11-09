import { describe, expect, it } from "vitest";
import { groupOf, parseTab, PROJECT_TABS, projectPath, TAB_GROUPS } from "./tabs";

describe("parseTab", () => {
  it("reads a known tab and falls back to the overview", () => {
    expect(parseTab("tasks")).toBe("tasks");
    expect(parseTab(undefined)).toBe("overview");
    expect(parseTab("nope")).toBe("overview");
  });
});

describe("groupOf", () => {
  it("finds the group holding a tab", () => {
    expect(groupOf("idea").label).toBe("Plan");
    expect(groupOf("time").label).toBe("Work");
    expect(groupOf("save-points").label).toBe("History");
  });

  it("puts every tab in exactly one group", () => {
    const grouped = TAB_GROUPS.flatMap((group) => group.tabs.map((tab) => tab.id));
    expect([...grouped].sort()).toEqual([...PROJECT_TABS].sort());
  });
});

describe("projectPath", () => {
  it("leaves the overview out of the url", () => {
    expect(projectPath("p1")).toBe("/projects/p1");
    expect(projectPath("p1", "save-points")).toBe("/projects/p1/save-points");
  });
});
