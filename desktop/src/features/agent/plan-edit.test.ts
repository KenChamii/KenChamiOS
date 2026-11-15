import { describe, expect, it } from "vitest";
import type { PlanProposal } from "@/lib/api/agent";
import { canCreate, countTasks, removeTask, renamePhase, renameTask } from "./plan-edit";

function plan(): PlanProposal {
  return {
    phases: [
      {
        title: "Setup",
        tasks: [
          { title: "Add table", body: "", acceptance: "" },
          { title: "Add command", body: "", acceptance: "" },
        ],
      },
      { title: "Screens", tasks: [{ title: "Build page", body: "", acceptance: "" }] },
    ],
  };
}

describe("plan edits", () => {
  it("counts tasks over all phases", () => {
    expect(countTasks(plan())).toBe(3);
  });

  it("renames one phase and one task without touching the original", () => {
    const original = plan();
    const edited = renameTask(renamePhase(original, 1, "UI"), 0, 1, "Add the command");
    expect(edited.phases[1].title).toBe("UI");
    expect(edited.phases[0].tasks[1].title).toBe("Add the command");
    expect(original.phases[1].title).toBe("Screens");
  });

  it("removes a task and keeps its phase", () => {
    const edited = removeTask(plan(), 0, 0);
    expect(edited.phases[0].tasks.map((t) => t.title)).toEqual(["Add command"]);
  });

  it("drops a phase when its last task is removed", () => {
    const edited = removeTask(plan(), 1, 0);
    expect(edited.phases.map((p) => p.title)).toEqual(["Setup"]);
  });

  it("can only create a plan with text in every title", () => {
    expect(canCreate(plan())).toBe(true);
    expect(canCreate({ phases: [] })).toBe(false);
    expect(canCreate(renamePhase(plan(), 0, "  "))).toBe(false);
    expect(canCreate(renameTask(plan(), 1, 0, ""))).toBe(false);
  });
});
