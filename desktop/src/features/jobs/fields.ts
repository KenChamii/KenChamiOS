import type { Job, JobInput } from "@/lib/api/jobs";

const pad = (value: number) => String(value).padStart(2, "0");

/** Unix ms to the value of a datetime-local input (local time), or "". */
export function toLocalInput(ms: number | null): string {
  if (ms === null) return "";
  const date = new Date(ms);
  const day = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  return `${day}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** The value of a datetime-local input to unix ms, or null when empty or not a date. */
export function fromLocalInput(value: string): number | null {
  if (!value) return null;
  const ms = new Date(value).getTime();
  return Number.isNaN(ms) ? null : ms;
}

export const EMPTY_JOB: JobInput = {
  company: "",
  role: "",
  url: "",
  location: "",
  salary: "",
  source: "other",
  applied_on: null,
  contact: "",
  resume_version: "",
  next_step: "",
  next_step_at: null,
  notes_md: "",
};

export function inputOf(job: Job): JobInput {
  const { company, role, url, location, salary, source, applied_on } = job;
  const { contact, resume_version, next_step, next_step_at, notes_md } = job;
  return {
    company,
    role,
    url,
    location,
    salary,
    source,
    applied_on,
    contact,
    resume_version,
    next_step,
    next_step_at,
    notes_md,
  };
}

/** A form can be saved when it has a company and a role. */
export function canSave(input: JobInput): boolean {
  return input.company.trim() !== "" && input.role.trim() !== "";
}
