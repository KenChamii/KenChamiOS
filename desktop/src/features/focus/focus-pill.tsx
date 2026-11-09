import { Pause, Play, Square, Timer } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { projectPath } from "@/features/projects/tabs";
import { useTasks } from "@/hooks/use-tasks";
import { usePauseTimer, useResumeTimer, useStopTimer, useTimer } from "@/hooks/use-timer";
import type { TimerView } from "@/lib/api/timer";
import { cn } from "@/lib/cn";
import { useOverlays } from "@/stores/overlays";
import { PHASE_LABELS, formatClock } from "./format";

/** The running timer in the sidebar, or a way to start one. */
export function FocusPill() {
  const timer = useTimer();
  const openPalette = useOverlays((state) => state.openPalette);

  if (timer.data) return <RunningPill view={timer.data} />;
  return (
    <button
      type="button"
      onClick={() => openPalette("focus")}
      className="flex w-full cursor-pointer items-center gap-3 rounded-xl bg-focus-soft px-3 py-2 text-sm font-medium text-focus transition-[filter] duration-(--duration-fast) hover:brightness-95"
    >
      <Timer size={18} />
      Start focus
    </button>
  );
}

function RunningPill({ view }: { view: TimerView }) {
  const tasks = useTasks(view.project_id);
  const pause = usePauseTimer();
  const resume = useResumeTimer();
  const stop = useStopTimer();
  const task = tasks.data?.find((item) => item.id === view.task_id);
  const paused = view.status === "paused";
  const focusing = view.phase === "focus";

  return (
    <div
      className={cn(
        "flex flex-col gap-1 rounded-2xl p-3 shadow-soft",
        focusing ? "bg-focus-soft" : "bg-workflow-soft",
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <span className={cn("text-label", focusing ? "text-focus" : "text-workflow")}>
          {PHASE_LABELS[view.phase]}
          {paused && " · Paused"}
        </span>
        <div className="flex">
          <Button
            size="icon"
            variant="ghost"
            className="h-7 w-7"
            aria-label={paused ? "Resume" : "Pause"}
            title={paused ? "Resume" : "Pause"}
            disabled={pause.isPending || resume.isPending}
            onClick={() => (paused ? resume.mutate(undefined) : pause.mutate(undefined))}
          >
            {paused ? <Play size={14} /> : <Pause size={14} />}
          </Button>
          <Button
            size="icon"
            variant="ghost"
            className="h-7 w-7"
            aria-label="Stop timer"
            title="Stop timer"
            disabled={stop.isPending}
            onClick={() => stop.mutate(undefined)}
          >
            <Square size={14} />
          </Button>
        </div>
      </div>
      <Link to={projectPath(view.project_id, "tasks")} className="flex flex-col">
        <span className="text-2xl font-semibold tabular-nums" aria-live="off">
          {formatClock(view.remaining_s)}
        </span>
        <span className="truncate text-xs text-muted-foreground">
          {task?.title ?? "Open project"}
        </span>
      </Link>
    </div>
  );
}
