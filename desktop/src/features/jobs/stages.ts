import type { JobSource, JobStage, JobView } from "@/lib/api/jobs";

export const STAGE_LABELS: Record<JobStage, string> = {
  saved: "Saved",
  applied: "Applied",
  screening: "Screening",
  interview: "Interview",
  offer: "Offer",
  accepted: "Accepted",
  rejected: "Rejected",
  withdrawn: "Withdrawn",
};

export const SOURCE_LABELS: Record<JobSource, string> = {
  linkedin: "LinkedIn",
  company_site: "Company site",
  job_board: "Job board",
  referral: "Referral",
  recruiter: "Recruiter",
  other: "Other",
};

/** The columns of the board; closed applications share the last one. */
export const BOARD_STAGES = ["saved", "applied", "screening", "interview", "offer"] as const;
export const CLOSED_STAGES: JobStage[] = ["accepted", "rejected", "withdrawn"];

export type Column = JobStage | "closed";

export const COLUMNS: { id: Column; label: string }[] = [
  ...BOARD_STAGES.map((stage) => ({ id: stage, label: STAGE_LABELS[stage] })),
  { id: "closed", label: "Closed" },
];

export function columnOf(stage: JobStage): Column {
  return CLOSED_STAGES.includes(stage) ? "closed" : stage;
}

export function groupByColumn(jobs: JobView[]): Record<Column, JobView[]> {
  const columns: Record<Column, JobView[]> = {
    saved: [],
    applied: [],
    screening: [],
    interview: [],
    offer: [],
    closed: [],
    accepted: [],
    rejected: [],
    withdrawn: [],
  };
  for (const job of jobs) columns[columnOf(job.stage)].push(job);
  return columns;
}

/** The stage a drop on a column means: the column itself, or Rejected for Closed. */
export function stageForDrop(column: Column, current: JobStage): JobStage {
  if (column !== "closed") return column;
  return CLOSED_STAGES.includes(current) ? current : "rejected";
}
