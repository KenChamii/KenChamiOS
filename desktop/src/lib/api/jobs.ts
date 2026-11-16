import { invoke } from "@tauri-apps/api/core";

export const JOB_STAGES = [
  "saved",
  "applied",
  "screening",
  "interview",
  "offer",
  "accepted",
  "rejected",
  "withdrawn",
] as const;
export type JobStage = (typeof JOB_STAGES)[number];

export const JOB_SOURCES = [
  "linkedin",
  "company_site",
  "job_board",
  "referral",
  "recruiter",
  "other",
] as const;
export type JobSource = (typeof JOB_SOURCES)[number];

export type Job = {
  id: string;
  created_at: number;
  updated_at: number;
  company: string;
  role: string;
  url: string;
  location: string;
  salary: string;
  source: JobSource;
  stage: JobStage;
  applied_on: string | null;
  contact: string;
  resume_version: string;
  next_step: string;
  next_step_at: number | null;
  notes_md: string;
  stage_changed_at: number;
};

/** A job with the flags the board shows: untouched for a week, next step due. */
export type JobView = Job & { stale: boolean; due: boolean };

export type JobInput = Omit<Job, "id" | "created_at" | "updated_at" | "stage" | "stage_changed_at">;

export type JobStats = {
  per_week: [string, number][];
  applied: number;
  responded: number;
  response_rate: number | null;
  time_per_stage: { stage: JobStage; avg_days: number }[];
};

export function listJobs() {
  return invoke<JobView[]>("jobs_list");
}

export function createJob(input: JobInput) {
  return invoke<Job>("jobs_create", { input });
}

export function updateJob(id: string, input: JobInput) {
  return invoke<Job>("jobs_update", { id, input });
}

export function moveJob(id: string, stage: JobStage, today: string) {
  return invoke<Job>("jobs_move", { id, stage, today });
}

export function deleteJob(id: string) {
  return invoke<void>("jobs_delete", { id });
}

export function getJobStats() {
  return invoke<JobStats>("jobs_stats");
}

export function listJobsNeedingAttention() {
  return invoke<JobView[]>("jobs_attention");
}
