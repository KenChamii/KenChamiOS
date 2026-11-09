import { useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { ChipGroup } from "@/components/ui/chips";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { PRIORITY_LABELS } from "@/features/tasks/meta";
import { newTask } from "@/features/tasks/new-task";
import { useCreateTask } from "@/hooks/use-tasks";
import type { ProjectSummary } from "@/lib/api/projects";
import { PRIORITIES, type Priority } from "@/lib/api/tasks";
import { errorMessage } from "@/lib/errors";
import { DUE_LABELS, dueAt, type DueChoice } from "./due";

type QuickTaskFormProps = {
  projects: ProjectSummary[];
  defaultProjectId: string | null;
  onAddIdea: () => void;
  onDone: () => void;
};

export function QuickTaskForm({
  projects,
  defaultProjectId,
  onAddIdea,
  onDone,
}: QuickTaskFormProps) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState("");
  const [projectId, setProjectId] = useState(defaultProjectId ?? projects[0]?.id ?? "");
  const [priority, setPriority] = useState<Priority>("medium");
  const [due, setDue] = useState<DueChoice>("none");
  const [picked, setPicked] = useState("");
  const create = useCreateTask(projectId);

  if (projects.length === 0) {
    return (
      <div className="flex flex-col items-center gap-3 py-6 text-center">
        <p className="text-sm text-muted-foreground">A task needs a project first.</p>
        <Button onClick={onAddIdea}>Add an idea</Button>
      </div>
    );
  }

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const input = {
      ...newTask(projectId, title.trim(), null),
      priority,
      due_at: dueAt(due, picked, Date.now()),
    };
    create.mutate(input, {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: ["dashboard"] });
        onDone();
      },
    });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Task">
        <Input value={title} onChange={(e) => setTitle(e.target.value)} autoFocus required />
      </Field>
      <Field label="Project">
        <Select
          value={projectId}
          options={projects.map((project) => ({ value: project.id, label: project.name }))}
          onChange={setProjectId}
        />
      </Field>
      <div className="flex flex-col gap-1.5">
        <span className="text-label text-muted-foreground">Priority</span>
        <ChipGroup
          aria-label="Priority"
          value={priority}
          options={PRIORITIES.map((value) => ({ value, label: PRIORITY_LABELS[value] }))}
          onChange={setPriority}
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <span className="text-label text-muted-foreground">Due</span>
        <ChipGroup
          aria-label="Due"
          value={due}
          options={(Object.keys(DUE_LABELS) as DueChoice[]).map((value) => ({
            value,
            label: DUE_LABELS[value],
          }))}
          onChange={setDue}
        />
        {due === "pick" && (
          <Input
            type="date"
            aria-label="Due date"
            className="w-48"
            value={picked}
            onChange={(e) => setPicked(e.target.value)}
          />
        )}
      </div>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
      <Button type="submit" disabled={create.isPending || !title.trim() || !projectId}>
        Add task
      </Button>
    </form>
  );
}
