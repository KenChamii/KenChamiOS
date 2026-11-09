import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useGitInfo } from "@/hooks/use-git";
import { useConfirmSessionNote } from "@/hooks/use-session-notes";
import { useTasks } from "@/hooks/use-tasks";
import type { ProjectSummary } from "@/lib/api/projects";
import type { NotePreview } from "@/lib/api/session-notes";
import { errorMessage } from "@/lib/errors";
import { commitLabel } from "./meta";
import {
  buildConfirm,
  DEFAULT_NOTE,
  initialChoices,
  type DoneChoice,
  type NextChoice,
} from "./session-note";

type SessionNotePreviewProps = {
  project: ProjectSummary;
  text: string;
  preview: NotePreview;
  onBack: () => void;
  onDone: () => void;
};

export function SessionNotePreview({
  project,
  text,
  preview,
  onBack,
  onDone,
}: SessionNotePreviewProps) {
  const confirm = useConfirmSessionNote(project.id);
  const git = useGitInfo(project.id);
  const tasks = useTasks(project.id);
  const [note, setNote] = useState(preview.parsed.note || DEFAULT_NOTE);
  const [done, setDone] = useState<DoneChoice[]>(() => initialChoices(preview).done);
  const [next, setNext] = useState<NextChoice[]>(() => initialChoices(preview).next);

  const openTasks = (tasks.data ?? []).filter((task) => task.status !== "done");
  const commit = git.data?.state === "repo" ? commitLabel(git.data.branch, git.data.head) : null;

  function handleConfirm() {
    const input = buildConfirm({ projectId: project.id, summary: text, note, preview, done, next });
    confirm.mutate(input, { onSuccess: onDone });
  }

  return (
    <div className="flex flex-col gap-4">
      <Field label="Note">
        <Textarea value={note} onChange={(event) => setNote(event.target.value)} />
      </Field>

      <section className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Done</h3>
        {done.length === 0 && <p className="text-sm text-muted-foreground">Nothing found.</p>}
        {done.map((choice, index) => (
          <div key={index} className="flex items-center gap-2 text-sm">
            <span className="flex-1">{choice.item}</span>
            <Select
              aria-label={`Task for ${choice.item}`}
              className="w-56"
              value={choice.taskId ?? ""}
              onChange={(event) =>
                setDone(
                  done.map((c, i) =>
                    i === index ? { ...c, taskId: event.target.value || null } : c,
                  ),
                )
              }
            >
              <option value="">No task</option>
              {openTasks.map((task) => (
                <option key={task.id} value={task.id}>
                  Mark done: {task.title}
                </option>
              ))}
            </Select>
          </div>
        ))}
      </section>

      <section className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Next</h3>
        {next.length === 0 && <p className="text-sm text-muted-foreground">Nothing found.</p>}
        {next.map((choice, index) => (
          <label key={index} className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={choice.createTask}
              onChange={(event) =>
                setNext(
                  next.map((c, i) =>
                    i === index ? { ...c, createTask: event.target.checked } : c,
                  ),
                )
              }
            />
            {choice.item}
            <span className="text-muted-foreground">(add as task)</span>
          </label>
        ))}
      </section>

      {preview.parsed.changed_files.length > 0 && (
        <section className="flex flex-col gap-1">
          <h3 className="text-sm font-medium">Changed files</h3>
          <ul className="font-mono text-xs text-muted-foreground">
            {preview.parsed.changed_files.map((file) => (
              <li key={file}>{file}</li>
            ))}
          </ul>
        </section>
      )}

      {commit && <p className="font-mono text-xs text-muted-foreground">Commit: {commit}</p>}
      {confirm.error && <p className="text-sm text-destructive">{errorMessage(confirm.error)}</p>}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onBack}>
          Back
        </Button>
        <Button disabled={confirm.isPending || !note.trim()} onClick={handleConfirm}>
          Create save point
        </Button>
      </div>
    </div>
  );
}
