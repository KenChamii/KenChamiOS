import { describe, expect, it } from "vitest";
import type { NotePreview } from "@/lib/api/session-notes";
import { buildConfirm, initialChoices } from "./session-note";

const preview: NotePreview = {
  parsed: {
    note: "Good session",
    done: ["login form", "tests"],
    next: ["deploy", "docs"],
    changed_files: ["src/a.ts"],
  },
  done_matches: [
    { item: "login form", task_id: "t1" },
    { item: "tests", task_id: null },
  ],
};

describe("initialChoices", () => {
  it("links done items to their match and turns every next item into a task", () => {
    expect(initialChoices(preview)).toEqual({
      done: [
        { item: "login form", taskId: "t1" },
        { item: "tests", taskId: null },
      ],
      next: [
        { item: "deploy", createTask: true },
        { item: "docs", createTask: true },
      ],
    });
  });
});

describe("buildConfirm", () => {
  it("collects the chosen tasks and new task titles", () => {
    const choices = initialChoices(preview);
    choices.next[1] = { item: "docs", createTask: false };

    const confirm = buildConfirm({
      projectId: "p1",
      summary: "raw",
      note: "  Good session ",
      preview,
      ...choices,
    });

    expect(confirm).toEqual({
      project_id: "p1",
      note: "Good session",
      summary_md: "raw",
      done: ["login form", "tests"],
      next: ["deploy", "docs"],
      changed_files: ["src/a.ts"],
      done_task_ids: ["t1"],
      new_task_titles: ["deploy"],
    });
  });

  it("links a task once even when two items point at it", () => {
    const confirm = buildConfirm({
      projectId: "p1",
      summary: "",
      note: "n",
      preview,
      done: [
        { item: "a", taskId: "t1" },
        { item: "b", taskId: "t1" },
      ],
      next: [],
    });
    expect(confirm.done_task_ids).toEqual(["t1"]);
  });
});
