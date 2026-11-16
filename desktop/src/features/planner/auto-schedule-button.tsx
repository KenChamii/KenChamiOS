import { Wand2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { useSetPlan } from "@/hooks/use-planner";
import type { FocusBlock, PlannedTask } from "@/lib/api/planner";
import { autoSchedule, waitingCount } from "./auto-schedule";

type Props = { planned: PlannedTask[]; focus: FocusBlock[] };

export function AutoScheduleButton({ planned, focus }: Props) {
  const setPlan = useSetPlan();
  const [message, setMessage] = useState<string | null>(null);
  const waiting = waitingCount(planned);

  async function run() {
    const { placed, unplaced } = autoSchedule(planned, focus, Date.now());
    for (const { task, plan } of placed) await setPlan.mutateAsync({ id: task.id, plan });
    setMessage(
      unplaced.length === 0
        ? `Placed ${placed.length}`
        : `Placed ${placed.length}, ${unplaced.length} did not fit`,
    );
  }

  return (
    <div className="flex items-center gap-3">
      <Button
        variant="outline"
        size="sm"
        disabled={waiting === 0 || setPlan.isPending}
        onClick={() => void run()}
      >
        <Wand2 size={14} />
        Auto-schedule
      </Button>
      {message && <span className="text-sm text-muted-foreground">{message}</span>}
    </div>
  );
}
