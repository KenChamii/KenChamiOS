import { describe, expect, it } from "vitest";
import type { JobView } from "@/lib/api/jobs";
import { columnOf, groupByColumn, stageForDrop } from "./stages";

const job = (id: string, stage: JobView["stage"]): JobView => ({
  id,
  created_at: 0,
  updated_at: 0,
  company: "A",
  role: "R",
  url: "",
  location: "",
  salary: "",
  source: "other",
  stage,
  applied_on: null,
  contact: "",
  resume_version: "",
  next_step: "",
  next_step_at: null,
  notes_md: "",
  stage_changed_at: 0,
  stale: false,
  due: false,
});

describe("columns", () => {
  it("puts the three closed stages in one column", () => {
    expect(columnOf("accepted")).toBe("closed");
    expect(columnOf("withdrawn")).toBe("closed");
    expect(columnOf("interview")).toBe("interview");
  });

  it("groups jobs by column", () => {
    const columns = groupByColumn([job("a", "saved"), job("b", "rejected"), job("c", "offer")]);
    expect(columns.saved.map((j) => j.id)).toEqual(["a"]);
    expect(columns.closed.map((j) => j.id)).toEqual(["b"]);
    expect(columns.offer.map((j) => j.id)).toEqual(["c"]);
  });
});

describe("stageForDrop", () => {
  it("keeps a closed stage when dropped on Closed, and defaults to rejected", () => {
    expect(stageForDrop("closed", "accepted")).toBe("accepted");
    expect(stageForDrop("closed", "interview")).toBe("rejected");
    expect(stageForDrop("applied", "saved")).toBe("applied");
  });
});
