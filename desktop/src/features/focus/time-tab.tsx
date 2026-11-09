import { Pencil, Timer, Trash2 } from "lucide-react";
import { useState } from "react";
import { Stat } from "@/components/layout/stat";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { useDeleteTimeEntry, useTimeEntries } from "@/hooks/use-time-entries";
import { useTasks } from "@/hooks/use-tasks";
import type { TimeEntry } from "@/lib/api/time-entries";
import { errorMessage } from "@/lib/errors";
import { DayChart } from "./day-chart";
import { EntryDialog } from "./entry-dialog";
import { formatDuration } from "./format";
import { summarize } from "./time-stats";

const KIND_LABELS = { focus: "Focus", break: "Break", manual: "Manual" } as const;

export function TimeTab({ projectId }: { projectId: string }) {
  const entries = useTimeEntries(projectId);
  const tasks = useTasks(projectId);
  const remove = useDeleteTimeEntry(projectId);
  const [now] = useState(() => Date.now());
  // "new" adds an entry, an entry edits it
  const [editing, setEditing] = useState<TimeEntry | "new" | null>(null);

  const taskList = tasks.data ?? [];
  const titleOf = (taskId: string | null) =>
    taskList.find((task) => task.id === taskId)?.title ?? "No task";
  const all = entries.data ?? [];
  const summary = summarize(all, now);
  const error = entries.error ?? remove.error;

  return (
    <div className="flex flex-col gap-4">
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

      {entries.isSuccess && all.length === 0 ? (
        <EmptyState
          icon={Timer}
          area="focus"
          line="No time logged yet. Start a timer on a task, or add time by hand."
          action={<Button onClick={() => setEditing("new")}>Add entry</Button>}
        />
      ) : (
        <>
          <div className="grid grid-cols-2 gap-4">
            <Card>
              <Stat label="Total" value={formatDuration(summary.totalSeconds)} />
            </Card>
            <Card>
              <Stat label="This week" value={formatDuration(summary.weekSeconds)} />
            </Card>
          </div>

          <Card className="flex flex-col gap-3">
            <h2 className="font-semibold tracking-tight">Last 14 days</h2>
            <DayChart days={summary.days} />
          </Card>

          <Card className="flex flex-col gap-3">
            <h2 className="font-semibold tracking-tight">Per task</h2>
            <ul className="flex flex-col gap-1 text-sm">
              {summary.perTask.map(({ taskId, seconds }) => (
                <li key={taskId ?? "none"} className="flex justify-between gap-4">
                  <span className="truncate">{titleOf(taskId)}</span>
                  <span className="text-muted-foreground">{formatDuration(seconds)}</span>
                </li>
              ))}
            </ul>
          </Card>

          <Card className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <h2 className="font-semibold tracking-tight">Entries</h2>
              <Button size="sm" variant="outline" onClick={() => setEditing("new")}>
                Add entry
              </Button>
            </div>
            <ul className="flex flex-col divide-y divide-border text-sm">
              {all.map((entry) => (
                <li key={entry.id} className="flex items-center gap-3 py-2">
                  <span className="w-24 text-muted-foreground">
                    {new Date(entry.started_at).toLocaleDateString()}
                  </span>
                  <span className="w-16 text-muted-foreground">{KIND_LABELS[entry.kind]}</span>
                  <span className="w-16">{formatDuration(entry.duration_s)}</span>
                  <span className="min-w-0 flex-1 truncate">
                    {titleOf(entry.task_id)}
                    {entry.note && <span className="text-muted-foreground"> - {entry.note}</span>}
                  </span>
                  <Button
                    size="icon"
                    variant="ghost"
                    aria-label="Edit entry"
                    title="Edit entry"
                    onClick={() => setEditing(entry)}
                  >
                    <Pencil size={16} />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    aria-label="Delete entry"
                    title="Delete entry"
                    disabled={remove.isPending}
                    onClick={() => remove.mutate(entry.id)}
                  >
                    <Trash2 size={16} />
                  </Button>
                </li>
              ))}
            </ul>
          </Card>
        </>
      )}

      <EntryDialog
        projectId={projectId}
        tasks={taskList}
        entry={editing === "new" ? null : editing}
        open={editing !== null}
        onClose={() => setEditing(null)}
      />
    </div>
  );
}
