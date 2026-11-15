import { Sparkles, X } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { projectPath } from "@/features/projects/tabs";
import { useCreatePlanTasks, usePlanPreview } from "@/hooks/use-agent-plan";
import type { PlanProposal } from "@/lib/api/agent";
import { errorMessage } from "@/lib/errors";
import { useToasts } from "@/stores/toasts";
import { canCreate, countTasks, removeTask, renamePhase, renameTask } from "./plan-edit";

export function PlanBreakdown({ projectId }: { projectId: string }) {
  const navigate = useNavigate();
  const preview = usePlanPreview(projectId);
  const create = useCreatePlanTasks(projectId);
  const [draft, setDraft] = useState<PlanProposal | null>(null);

  function ask() {
    create.reset();
    preview.mutate(undefined, { onSuccess: setDraft });
  }

  function approve(plan: PlanProposal) {
    create.mutate(plan, {
      onSuccess: (tasks) => {
        setDraft(null);
        useToasts.getState().show(`${tasks.length} tasks created`);
        navigate(projectPath(projectId, "tasks"));
      },
    });
  }

  const error = preview.error ?? create.error;

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h2 className="flex items-center gap-2 font-semibold tracking-tight">
            <Sparkles size={16} />
            Break down with Claude
          </h2>
          <p className="text-sm text-muted-foreground">
            Turns the saved plan into phases and tasks. You review them before anything is created.
          </p>
        </div>
        {!draft && (
          <Button onClick={ask} disabled={preview.isPending}>
            {preview.isPending ? "Reading the plan..." : "Break down plan"}
          </Button>
        )}
      </div>
      {preview.isPending && (
        <p className="text-sm text-muted-foreground">This can take a minute.</p>
      )}
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {draft && (
        <PlanDraft
          plan={draft}
          busy={create.isPending}
          onChange={setDraft}
          onCancel={() => setDraft(null)}
          onApprove={() => approve(draft)}
        />
      )}
    </Card>
  );
}

type PlanDraftProps = {
  plan: PlanProposal;
  busy: boolean;
  onChange: (plan: PlanProposal) => void;
  onCancel: () => void;
  onApprove: () => void;
};

function PlanDraft({ plan, busy, onChange, onCancel, onApprove }: PlanDraftProps) {
  const total = countTasks(plan);
  return (
    <div className="flex flex-col gap-5">
      {plan.phases.map((phase, p) => (
        <section key={p} className="flex flex-col gap-2">
          <Input
            aria-label={`Phase ${p + 1} title`}
            value={phase.title}
            onChange={(e) => onChange(renamePhase(plan, p, e.target.value))}
            className="font-semibold"
          />
          <ul className="flex flex-col gap-2 pl-4">
            {phase.tasks.map((task, t) => (
              <li key={t} className="flex items-start gap-2">
                <div className="flex min-w-0 flex-1 flex-col gap-1">
                  <Input
                    aria-label={`Task ${t + 1} of phase ${p + 1}`}
                    value={task.title}
                    onChange={(e) => onChange(renameTask(plan, p, t, e.target.value))}
                  />
                  {task.acceptance && (
                    <p className="px-1 text-label text-muted-foreground">{task.acceptance}</p>
                  )}
                </div>
                <Button
                  size="icon"
                  variant="ghost"
                  aria-label={`Remove ${task.title || "task"}`}
                  onClick={() => onChange(removeTask(plan, p, t))}
                >
                  <X size={16} />
                </Button>
              </li>
            ))}
          </ul>
        </section>
      ))}
      <div className="flex items-center gap-2">
        <Button onClick={onApprove} disabled={busy || !canCreate(plan)}>
          {busy ? "Creating..." : `Create ${total} ${total === 1 ? "task" : "tasks"}`}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={busy}>
          Cancel
        </Button>
      </div>
    </div>
  );
}
