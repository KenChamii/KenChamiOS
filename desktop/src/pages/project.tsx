import { ArrowLeft } from "lucide-react";
import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Tabs } from "@/components/ui/tabs";
import { DocTab } from "@/features/docs/doc-tab";
import { STAGE_LABELS } from "@/features/projects/meta";
import { OverviewTab } from "@/features/projects/overview-tab";
import { SavePointsTab } from "@/features/savepoints/save-points-tab";
import { TasksTab } from "@/features/tasks/tasks-tab";
import { useProject } from "@/hooks/use-projects";
import { errorMessage } from "@/lib/errors";

const TABS = [
  { id: "overview", label: "Overview" },
  { id: "idea", label: "Idea" },
  { id: "plan", label: "Plan" },
  { id: "tasks", label: "Tasks" },
  { id: "save-points", label: "Save points" },
  { id: "time", label: "Time" },
] as const;

type TabId = (typeof TABS)[number]["id"];

export function ProjectPage() {
  const { id = "" } = useParams();
  const project = useProject(id);
  const [tab, setTab] = useState<TabId>("overview");

  if (project.error) {
    return <p className="text-sm text-destructive">{errorMessage(project.error)}</p>;
  }
  if (!project.data) return null;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2">
        <Link
          to="/projects"
          className="flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft size={14} />
          Projects
        </Link>
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-semibold">{project.data.name}</h1>
          <Badge>{STAGE_LABELS[project.data.stage]}</Badge>
          {project.data.archived && <Badge>Archived</Badge>}
        </div>
      </div>

      <Tabs tabs={[...TABS]} value={tab} onChange={setTab} />

      {tab === "overview" && <OverviewTab project={project.data} />}
      {tab === "idea" && <DocTab projectId={id} kind="idea" />}
      {tab === "plan" && <DocTab projectId={id} kind="plan" />}
      {tab === "tasks" && <TasksTab projectId={id} />}
      {tab === "save-points" && <SavePointsTab project={project.data} />}
      {tab === "time" && (
        <p className="text-muted-foreground">Time tracking is not available yet.</p>
      )}
    </div>
  );
}
