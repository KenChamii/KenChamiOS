import { useState, type FormEvent } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { PRIORITY_LABELS } from "@/features/tasks/meta";
import { useCreateTask } from "@/hooks/use-tasks";
import type { ProjectSummary } from "@/lib/api/projects";
import { fromDateInput, toDateInput } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { INBOX_ID } from "@/lib/inbox";
import { ruleLabel } from "@/lib/recurrence";
import { parseQuickAdd } from "./parse";
import { matchProject, taskFromQuickAdd } from "./resolve";

type QuickTaskFormProps = {
  projects: ProjectSummary[];
  defaultProjectId: string | null;
  initialText: string;
  onDone: () => void;
};

export function QuickTaskForm({
  projects,
  defaultProjectId,
  initialText,
  onDone,
}: QuickTaskFormProps) {
  const [text, setText] = useState(initialText);
  const [chosenId, setChosenId] = useState(defaultProjectId ?? INBOX_ID);
  const [today] = useState(() => toDateInput(Date.now()));
  const parsed = parseQuickAdd(text, today);
  const tagged = parsed.project === null ? null : matchProject(parsed.project, projects);
  const projectId = tagged?.id ?? chosenId;
  const create = useCreateTask(projectId);

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    create.mutate(taskFromQuickAdd(parsed, projectId), { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Task">
        <Input
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Pay rent friday !high #home every month"
          autoFocus
          required
        />
      </Field>
      <div className="flex min-h-6 flex-wrap gap-1.5" aria-live="polite">
        {parsed.due && (
          <Badge>Due {new Date(fromDateInput(parsed.due) ?? 0).toLocaleDateString()}</Badge>
        )}
        {parsed.priority && <Badge>{PRIORITY_LABELS[parsed.priority]} priority</Badge>}
        {parsed.recurrence && <Badge>{ruleLabel(parsed.recurrence)}</Badge>}
        {tagged && <Badge>{tagged.name}</Badge>}
        {parsed.project !== null && !tagged && (
          <Badge className="text-destructive">No project named #{parsed.project}</Badge>
        )}
      </div>
      {!tagged && (
        <Field label="Project">
          <Select
            value={chosenId}
            options={[
              { value: INBOX_ID, label: "Inbox" },
              ...projects.map((project) => ({ value: project.id, label: project.name })),
            ]}
            onChange={setChosenId}
          />
        </Field>
      )}
      <p className="text-xs text-muted-foreground">
        Type dates like tomorrow or friday, !high, #project and every week.
      </p>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
      <Button type="submit" disabled={create.isPending || !parsed.title}>
        Add task
      </Button>
    </form>
  );
}
