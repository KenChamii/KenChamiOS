import { Inbox, X } from "lucide-react";
import { useState, type FormEvent } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { parseQuickAdd } from "@/features/quick-add/parse";
import { matchProject, taskFromQuickAdd } from "@/features/quick-add/resolve";
import { TaskDrawer } from "@/features/tasks/task-drawer";
import { TaskMetaBadges } from "@/features/tasks/task-meta-badges";
import { useProjects } from "@/hooks/use-projects";
import {
  useCreateTask,
  useDeleteTask,
  useMoveTask,
  useMoveTaskToProject,
  useTasks,
} from "@/hooks/use-tasks";
import type { ProjectSummary } from "@/lib/api/projects";
import type { Task } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
import { toDateInput } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { INBOX_ID } from "@/lib/inbox";
import { stagger } from "@/lib/motion";

const ACTIVE_PROJECTS = { search: null, tag: null, include_archived: false };

export function InboxPage() {
  const tasks = useTasks(INBOX_ID);
  const projects = useProjects(ACTIVE_PROJECTS);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const all = tasks.data ?? [];
  // open tasks first, done ones stay until they are moved or deleted
  const topLevel = all
    .filter((task) => task.parent_id === null)
    .sort((a, b) => Number(a.status === "done") - Number(b.status === "done"));
  const selected = all.find((task) => task.id === selectedId) ?? null;
  const subtasks = selectedId ? all.filter((task) => task.parent_id === selectedId) : [];

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title="Inbox" subtitle="Capture now, sort into projects later" icon={Inbox} />
      <CaptureForm projects={projects.data ?? []} />
      {tasks.error && <p className="text-sm text-destructive">{errorMessage(tasks.error)}</p>}
      {tasks.data && topLevel.length === 0 ? (
        <EmptyState icon={Inbox} line="Nothing waiting. Capture anything with Ctrl+N." />
      ) : (
        <ul className="flex flex-col divide-y divide-border rounded-2xl bg-card shadow-soft">
          {topLevel.map((task, index) => (
            <InboxRow
              key={task.id}
              task={task}
              index={index}
              projects={projects.data ?? []}
              onSelect={() => setSelectedId(task.id)}
            />
          ))}
        </ul>
      )}
      <TaskDrawer task={selected} subtasks={subtasks} onClose={() => setSelectedId(null)} />
    </div>
  );
}

// a #project in the line files the task right away, everything else lands here
function CaptureForm({ projects }: { projects: ProjectSummary[] }) {
  const [text, setText] = useState("");
  const [today] = useState(() => toDateInput(Date.now()));
  const parsed = parseQuickAdd(text, today);
  const target = (parsed.project && matchProject(parsed.project, projects)?.id) || INBOX_ID;
  const create = useCreateTask(target);

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!parsed.title) return;
    create.mutate(taskFromQuickAdd(parsed, target), { onSuccess: () => setText("") });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-2">
      <div className="flex gap-2">
        <Input
          aria-label="Capture a task"
          placeholder="Call the bank tomorrow !high"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        <Button type="submit" disabled={create.isPending || !parsed.title}>
          Capture
        </Button>
      </div>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
    </form>
  );
}

type InboxRowProps = {
  task: Task;
  index: number;
  projects: ProjectSummary[];
  onSelect: () => void;
};

function InboxRow({ task, index, projects, onSelect }: InboxRowProps) {
  const move = useMoveTask(INBOX_ID);
  const file = useMoveTaskToProject(INBOX_ID);
  const remove = useDeleteTask(INBOX_ID);
  const done = task.status === "done";
  const error = move.error ?? file.error ?? remove.error;

  return (
    <li className="stagger-in flex flex-col gap-1 px-4 py-2" style={stagger(index)}>
      <div className="flex items-center gap-3">
        <Checkbox
          aria-label={`Complete ${task.title}`}
          checked={done}
          onCheckedChange={(checked) =>
            move.mutate({ task, status: checked ? "done" : "todo", beforeId: null })
          }
        />
        <button
          type="button"
          onClick={onSelect}
          className={cn("flex-1 text-left text-sm", done && "text-muted-foreground line-through")}
        >
          {task.title}
        </button>
        <TaskMetaBadges task={task} />
        <Select
          aria-label={`Move ${task.title} to a project`}
          className="w-44"
          value=""
          disabled={projects.length === 0 || file.isPending}
          options={[
            { value: "", label: projects.length ? "Move to…" : "No projects yet" },
            ...projects.map((project) => ({ value: project.id, label: project.name })),
          ]}
          onChange={(projectId) => projectId && file.mutate({ id: task.id, projectId })}
        />
        <Button
          variant="ghost"
          size="icon"
          aria-label={`Delete ${task.title}`}
          onClick={() => remove.mutate(task.id)}
        >
          <X size={14} />
        </Button>
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </li>
  );
}
