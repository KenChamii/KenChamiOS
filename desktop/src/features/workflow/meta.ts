import type { TemplateKind } from "@/lib/api/templates";

export const TEMPLATE_TITLES: Record<TemplateKind, string> = {
  brief: "Project brief",
  task_prompt: "Task prompt",
  portfolio: "Portfolio",
};

export const TEMPLATE_HELP: Record<TemplateKind, string> = {
  brief:
    "Variables: project, docs.idea, docs.plan, tasks (open), save_point, git. Written into the target file of each project.",
  task_prompt:
    "Variables: task (title, body, acceptance, related_files), project, git. Used by Copy as prompt.",
  portfolio: "",
};
