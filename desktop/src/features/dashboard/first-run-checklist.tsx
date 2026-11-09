import { Check } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useMedia } from "@/hooks/use-media";
import { useProjects } from "@/hooks/use-projects";
import { useSyncStatus } from "@/hooks/use-sync";
import { cn } from "@/lib/cn";
import { useOverlays } from "@/stores/overlays";
import { firstRunSteps, isFirstRun, type StepId } from "./first-run";

const STEP_TEXT: Record<StepId, { title: string; hint: string }> = {
  project: { title: "Create a project", hint: "Anything you are building, from an idea up." },
  task: { title: "Add a task", hint: "The next small step on a project." },
  media: { title: "Add a title to your library", hint: "A show, game, book or album." },
  phone: { title: "Pair your phone", hint: "Optional. Syncs over your home Wi-Fi." },
};

/** Shown on a fresh install, until there is a project or a title. */
export function FirstRunChecklist() {
  const projects = useProjects({ search: null, tag: null, include_archived: true });
  const media = useMedia();
  const sync = useSyncStatus();
  const openQuickAdd = useOverlays((state) => state.openQuickAdd);
  if (!projects.data || !media.data) return null;

  const counts = {
    projects: projects.data.length,
    tasks: projects.data.reduce((sum, project) => sum + project.tasks_total, 0),
    media: media.data.length,
    devices: sync.data?.devices.length ?? 0,
  };
  if (!isFirstRun(counts)) return null;

  const actions: Record<StepId, ReactNode> = {
    project: <Button onClick={() => openQuickAdd("idea")}>Create</Button>,
    task: (
      <Button
        variant="outline"
        disabled={counts.projects === 0}
        onClick={() => openQuickAdd("task")}
      >
        Add
      </Button>
    ),
    media: (
      <Button variant="outline" asChild>
        <Link to="/media">Open library</Link>
      </Button>
    ),
    phone: (
      <Button variant="ghost" asChild>
        <Link to="/settings/sync">Pair</Link>
      </Button>
    ),
  };

  return (
    <Card className="flex flex-col gap-4 p-6">
      <div className="flex flex-col gap-0.5">
        <h2 className="text-lg font-semibold tracking-tight">Get started</h2>
        <p className="text-sm text-muted-foreground">Four small steps to make this place yours.</p>
      </div>
      <ol className="flex flex-col divide-y divide-border">
        {firstRunSteps(counts).map((step, index) => (
          <li key={step.id} className="flex items-center gap-4 py-3">
            <span
              className={cn(
                "flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted text-sm font-medium",
                step.done && "bg-primary text-primary-foreground",
              )}
            >
              {step.done ? <Check size={14} strokeWidth={3} className="animate-tick" /> : index + 1}
            </span>
            <div className="flex min-w-0 flex-1 flex-col">
              <span className={cn("text-sm font-medium", step.done && "line-through")}>
                {STEP_TEXT[step.id].title}
              </span>
              <span className="text-xs text-muted-foreground">{STEP_TEXT[step.id].hint}</span>
            </div>
            {!step.done && actions[step.id]}
          </li>
        ))}
      </ol>
    </Card>
  );
}
