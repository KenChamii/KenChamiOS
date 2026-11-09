import { describe, expect, it } from "vitest";
import { firstRunSteps, isFirstRun } from "./first-run";

const EMPTY = { projects: 0, tasks: 0, media: 0, devices: 0 };

describe("isFirstRun", () => {
  it("is true with no projects and no media", () => {
    expect(isFirstRun(EMPTY)).toBe(true);
    expect(isFirstRun({ ...EMPTY, devices: 1 })).toBe(true);
  });

  it("is false once there is a project or a title", () => {
    expect(isFirstRun({ ...EMPTY, projects: 1 })).toBe(false);
    expect(isFirstRun({ ...EMPTY, media: 3 })).toBe(false);
  });
});

describe("firstRunSteps", () => {
  it("lists the four steps in order, none done on an empty install", () => {
    expect(firstRunSteps(EMPTY)).toEqual([
      { id: "project", done: false },
      { id: "task", done: false },
      { id: "media", done: false },
      { id: "phone", done: false },
    ]);
  });

  it("marks each step done from its own count", () => {
    const steps = firstRunSteps({ projects: 1, tasks: 0, media: 2, devices: 1 });
    expect(steps.map((step) => step.done)).toEqual([true, false, true, true]);
  });
});
