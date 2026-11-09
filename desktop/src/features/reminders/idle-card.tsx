import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { STAGE_LABELS } from "@/features/projects/meta";
import { useIdleProjects, useSnoozeProject } from "@/hooks/use-reminders";
import { errorMessage } from "@/lib/errors";
import { idleLabel } from "./idle-days";

/** Projects that stopped without a save point; shown only while there are any. */
export function IdleCard() {
  const idle = useIdleProjects();
  const snooze = useSnoozeProject();
  const error = idle.error ?? snooze.error;
  if (!error && !idle.data?.length) return null;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Worth a save point</h2>
      <p className="text-sm text-muted-foreground">
        These projects went quiet after your last save point. Write down where you stopped.
      </p>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <ul className="flex flex-col gap-2">
        {idle.data?.map((project) => (
          <li key={project.project_id} className="flex items-center justify-between gap-3">
            <Link
              to={`/projects/${project.project_id}`}
              className="flex min-w-0 flex-col text-sm hover:underline"
            >
              <span className="flex items-center gap-2">
                <span className="truncate font-medium">{project.name}</span>
                <Badge>{STAGE_LABELS[project.stage]}</Badge>
              </span>
              <span className="text-xs text-muted-foreground">{idleLabel(project.idle_days)}</span>
            </Link>
            <Button
              variant="outline"
              disabled={snooze.isPending}
              onClick={() => snooze.mutate(project.project_id)}
            >
              Snooze
            </Button>
          </li>
        ))}
      </ul>
    </Card>
  );
}
