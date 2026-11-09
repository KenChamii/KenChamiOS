import type { NotePreview, SessionConfirm } from "@/lib/api/session-notes";

export type DoneChoice = { item: string; taskId: string | null };
export type NextChoice = { item: string; createTask: boolean };

export const DEFAULT_NOTE = "Session summary";

/** Done items start linked to their suggested task, next items start as new tasks. */
export function initialChoices(preview: NotePreview): { done: DoneChoice[]; next: NextChoice[] } {
  return {
    done: preview.done_matches.map(({ item, task_id }) => ({ item, taskId: task_id })),
    next: preview.parsed.next.map((item) => ({ item, createTask: true })),
  };
}

type BuildArgs = {
  projectId: string;
  summary: string;
  note: string;
  preview: NotePreview;
  done: DoneChoice[];
  next: NextChoice[];
};

export function buildConfirm({
  projectId,
  summary,
  note,
  preview,
  done,
  next,
}: BuildArgs): SessionConfirm {
  const taskIds = done.flatMap((choice) => (choice.taskId ? [choice.taskId] : []));
  return {
    project_id: projectId,
    note: note.trim(),
    summary_md: summary,
    done: done.map((choice) => choice.item),
    next: next.map((choice) => choice.item),
    changed_files: preview.parsed.changed_files,
    done_task_ids: [...new Set(taskIds)],
    new_task_titles: next.filter((choice) => choice.createTask).map((choice) => choice.item),
  };
}
