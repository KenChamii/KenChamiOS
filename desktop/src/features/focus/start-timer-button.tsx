import { Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useStartTimer, useTimer } from "@/hooks/use-timer";
import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";

type StartTimerButtonProps = { taskId: string; showLabel?: boolean };

export function StartTimerButton({ taskId, showLabel = false }: StartTimerButtonProps) {
  const timer = useTimer();
  const start = useStartTimer();
  const busy = timer.data != null;
  const label = start.error
    ? errorMessage(start.error)
    : busy
      ? "A timer is already running"
      : "Start focus timer";

  return (
    <Button
      type="button"
      variant={showLabel ? "outline" : "ghost"}
      size={showLabel ? "sm" : "icon"}
      className={cn(start.error && "text-destructive")}
      aria-label={label}
      title={label}
      disabled={busy || start.isPending}
      // the card and the row behind the button open the task on click
      onClick={(event) => {
        event.stopPropagation();
        start.mutate(taskId);
      }}
    >
      <Play size={16} />
      {showLabel && "Start timer"}
    </Button>
  );
}
