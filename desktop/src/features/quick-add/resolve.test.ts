import { describe, expect, it } from "vitest";
import type { ProjectSummary } from "@/lib/api/projects";
import { matchProject, taskFromQuickAdd } from "./resolve";

function project(id: string, name: string, slug: string): ProjectSummary {
  return {
    id,
    created_at: 0,
    name,
    slug,
    stage: "idea",
    description: "",
    repo_path: null,
    color: null,
    tags: [],
    archived: false,
    finished_at: null,
    tasks_total: 0,
    tasks_done: 0,
    last_activity_at: 0,
  };
}

describe("matchProject", () => {
  const projects = [project("a", "Team Sync", "team-sync"), project("b", "Home", "home")];

  it("finds a project by slug or name, ignoring case", () => {
    expect(matchProject("TEAM-SYNC", projects)?.id).toBe("a");
    expect(matchProject("home", projects)?.id).toBe("b");
    expect(matchProject("work", projects)).toBeNull();
  });
});

describe("taskFromQuickAdd", () => {
  it("turns a parsed line into a task, medium priority by default", () => {
    const task = taskFromQuickAdd(
      {
        title: "Pay rent",
        due: "2026-12-01",
        priority: null,
        project: null,
        recurrence: "monthly:1",
      },
      "p",
    );
    expect(task).toMatchObject({
      project_id: "p",
      title: "Pay rent",
      priority: "medium",
      due_at: new Date(2026, 11, 1).getTime(),
      recurrence: "monthly:1",
    });
  });
});
