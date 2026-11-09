import { Pause, Play, Square } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { useTasks } from "@/hooks/use-tasks";
import { usePauseTimer, useResumeTimer, useStopTimer, useTimer } from "@/hooks/use-timer";
import type { TimerView } from "@/lib/api/timer";
import { cn } from "@/lib/cn";
import { PHASE_LABELS, formatClock } from "./format";

export function TimerBar() {
  const timer = useTimer();
  return timer.data ? <ActiveTimer view={timer.data} /> : null;
}

function ActiveTimer({ view }: { view: TimerView }) {
  const tasks = useTasks(view.project_id);
  const pause = usePauseTimer();
  const resume = useResumeTimer();
  const stop = useStopTimer();
  const task = tasks.data?.find((item) => item.id === view.task_id);
  const paused = view.status === "paused";

  return (
    <div
      className={cn(
        "flex items-center gap-4 border-b border-border px-6 py-2 text-sm",
        view.phase === "focus" ? "bg-card" : "bg-muted",
      )}
    >
      <span className="font-medium">{PHASE_LABELS[view.phase]}</span>
      <span className="font-mono text-lg tabular-nums" aria-live="off">
        {formatClock(view.remaining_s)}
      </span>
      {paused && <span className="text-muted-foreground">Paused</span>}
      <Link
        to={`/projects/${view.project_id}`}
        className="min-w-0 flex-1 truncate text-muted-foreground hover:text-foreground"
      >
        {task?.title ?? "Open project"}
      </Link>
      <Button
        size="icon"
        variant="ghost"
        aria-label={paused ? "Resume" : "Pause"}
        title={paused ? "Resume" : "Pause"}
        disabled={pause.isPending || resume.isPending}
        onClick={() => (paused ? resume.mutate(undefined) : pause.mutate(undefined))}
      >
        {paused ? <Play size={16} /> : <Pause size={16} />}
      </Button>
      <Button
        size="icon"
        variant="ghost"
        aria-label="Stop timer"
        title="Stop timer"
        disabled={stop.isPending}
        onClick={() => stop.mutate(undefined)}
      >
        <Square size={16} />
      </Button>
    </div>
  );
}
