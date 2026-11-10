import { Check } from "lucide-react";
import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { SavePointDialog } from "@/features/savepoints/save-point-dialog";
import { useAppendDailyNote } from "@/hooks/use-daily-notes";
import { usePlanned, useSetPlan } from "@/hooks/use-planner";
import { useProject } from "@/hooks/use-projects";
import { useMoveTask } from "@/hooks/use-tasks";
import { NO_PLAN, type PlannedTask } from "@/lib/api/planner";
import { addDays, type Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";
import { INBOX_ID } from "@/lib/inbox";
import { touchedProjects } from "./review";

/** The evening review: settle today's plan, write a line, leave a save point. */
export function ReviewDay({ today, onClose }: { today: Day; onClose: () => void }) {
  const planned = usePlanned(today, today);
  const tasks = planned.data ?? [];
  const projects = touchedProjects(tasks, INBOX_ID);

  return (
    <div className="flex flex-col gap-5">
      {planned.error && <p className="text-sm text-destructive">{errorMessage(planned.error)}</p>}
      <section className="flex flex-col gap-2">
        <h3 className="text-label uppercase tracking-wider text-muted-foreground">Today's plan</h3>
        {planned.data?.length === 0 && (
          <p className="text-sm text-muted-foreground">Nothing was planned for today.</p>
        )}
        <ul className="flex flex-col gap-1">
          {tasks.map((task) => (
            <ReviewRow key={task.id} task={task} today={today} />
          ))}
        </ul>
      </section>
      <NoteForm today={today} onWritten={onClose} />
      {projects.length > 0 && (
        <section className="flex flex-col gap-2">
          <h3 className="text-label uppercase tracking-wider text-muted-foreground">Save points</h3>
          <div className="flex flex-wrap gap-2">
            {projects.map((project) => (
              <SavePointButton key={project.id} projectId={project.id} name={project.name} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function ReviewRow({ task, today }: { task: PlannedTask; today: Day }) {
  const move = useMoveTask(task.project_id);
  const setPlan = useSetPlan();
  const pending = move.isPending || setPlan.isPending;
  const error = move.error ?? setPlan.error;

  return (
    <li className="flex flex-col gap-1">
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate text-sm">
          {task.title}
          <span className="text-muted-foreground"> · {task.project_name}</span>
        </span>
        {task.status === "done" ? (
          <span className="flex items-center gap-1 text-sm text-muted-foreground">
            <Check size={14} />
            Done
          </span>
        ) : (
          <>
            <Button
              size="sm"
              disabled={pending}
              onClick={() => move.mutate({ task, status: "done", beforeId: null })}
            >
              Done
            </Button>
            <Button
              size="sm"
              variant="outline"
              disabled={pending}
              onClick={() =>
                setPlan.mutate({
                  id: task.id,
                  plan: {
                    planned_on: addDays(today, 1),
                    scheduled_at: null,
                    duration_min: task.duration_min,
                  },
                })
              }
            >
              Tomorrow
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={pending}
              onClick={() => setPlan.mutate({ id: task.id, plan: NO_PLAN })}
            >
              Drop
            </Button>
          </>
        )}
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </li>
  );
}

function NoteForm({ today, onWritten }: { today: Day; onWritten: () => void }) {
  const append = useAppendDailyNote();
  const [text, setText] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    append.mutate({ day: today, text }, { onSuccess: () => setText("") });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-2">
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="text-label uppercase tracking-wider text-muted-foreground">Note</h3>
        <Link
          to={`/notes?day=${today}`}
          onClick={onWritten}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          Open today's note
        </Link>
      </div>
      <Textarea
        aria-label="Note for today"
        placeholder="How did today go? Added to today's note."
        value={text}
        onChange={(event) => setText(event.target.value)}
      />
      <div className="flex items-center gap-3">
        <Button type="submit" variant="outline" disabled={append.isPending || !text.trim()}>
          Add to today's note
        </Button>
        {append.isSuccess && <span className="text-sm text-muted-foreground">Added</span>}
      </div>
      {append.error && <p className="text-sm text-destructive">{errorMessage(append.error)}</p>}
    </form>
  );
}

function SavePointButton({ projectId, name }: { projectId: string; name: string }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="outline" size="sm" onClick={() => setOpen(true)}>
        {name}
      </Button>
      {open && <ProjectSavePoint projectId={projectId} onClose={() => setOpen(false)} />}
    </>
  );
}

// the save point form needs the full project, loaded only once the button is used
function ProjectSavePoint({ projectId, onClose }: { projectId: string; onClose: () => void }) {
  const project = useProject(projectId);
  if (project.error)
    return <p className="text-sm text-destructive">{errorMessage(project.error)}</p>;
  if (!project.data) return null;
  return (
    <SavePointDialog
      project={project.data}
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    />
  );
}
