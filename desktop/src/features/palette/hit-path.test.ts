import { describe, expect, it } from "vitest";
import type { SearchHit } from "@/lib/api/search";
import { INBOX_ID } from "@/lib/inbox";
import { hitPath } from "./hit-path";

function hit(kind: SearchHit["kind"], projectId: string | null): SearchHit {
  return { kind, id: "x", project_id: projectId, project_name: null, title: "t", snippet: "" };
}

describe("hitPath", () => {
  it("opens the tab that holds the hit", () => {
    expect(hitPath(hit("task", "p"))).toBe("/projects/p/tasks");
    expect(hitPath(hit("plan", "p"))).toBe("/projects/p/plan");
    expect(hitPath(hit("save_point", "p"))).toBe("/projects/p/save-points");
    expect(hitPath(hit("prompt", "p"))).toBe("/projects/p/prompts");
  });

  it("sends inbox tasks to the inbox and global prompts to the workflow page", () => {
    expect(hitPath(hit("task", INBOX_ID))).toBe("/inbox");
    expect(hitPath(hit("prompt", null))).toBe("/workflow");
  });
});
