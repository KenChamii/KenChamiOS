import { ArrowLeft } from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { ChipGroup } from "@/components/ui/chips";
import { Tabs } from "@/components/ui/tabs";
import { DocTab } from "@/features/docs/doc-tab";
import { TimeTab } from "@/features/focus/time-tab";
import { PortfolioTab } from "@/features/portfolio/portfolio-tab";
import { OverviewTab } from "@/features/projects/overview-tab";
import { StageBadge } from "@/features/projects/stage-badge";
import { groupOf, parseTab, projectPath, TAB_GROUPS } from "@/features/projects/tabs";
import { SavePointsTab } from "@/features/savepoints/save-points-tab";
import { TasksTab } from "@/features/tasks/tasks-tab";
import { ProjectPromptsTab } from "@/features/workflow/project-prompts-tab";
import { useProject } from "@/hooks/use-projects";
import { errorMessage } from "@/lib/errors";

export function ProjectPage() {
  const params = useParams();
  const navigate = useNavigate();
  const id = params.id ?? "";
  const tab = parseTab(params.tab);
  const group = groupOf(tab);
  const project = useProject(id);

  if (project.error) {
    return <p className="text-sm text-destructive">{errorMessage(project.error)}</p>;
  }
  if (!project.data) return null;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-2">
        <Link
          to="/projects"
          className="flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft size={14} />
          Projects
        </Link>
        <div className="flex flex-wrap items-center gap-3">
          <h1 className="text-display">{project.data.name}</h1>
          <StageBadge stage={project.data.stage} />
          {project.data.archived && <Badge>Archived</Badge>}
        </div>
      </div>

      <div className="flex flex-col gap-3">
        <Tabs
          tabs={TAB_GROUPS.map(({ id: groupId, label }) => ({ id: groupId, label }))}
          value={group.id}
          onChange={(groupId) => {
            const next = TAB_GROUPS.find((candidate) => candidate.id === groupId);
            if (next) navigate(projectPath(id, next.tabs[0].id));
          }}
        />
        {group.tabs.length > 1 && (
          <ChipGroup
            aria-label={group.label}
            options={group.tabs.map((item) => ({ value: item.id, label: item.label }))}
            value={tab}
            onChange={(next) => navigate(projectPath(id, next))}
          />
        )}
      </div>

      {tab === "overview" && <OverviewTab project={project.data} />}
      {tab === "idea" && <DocTab projectId={id} kind="idea" />}
      {tab === "plan" && <DocTab projectId={id} kind="plan" />}
      {tab === "tasks" && <TasksTab projectId={id} />}
      {tab === "time" && <TimeTab projectId={id} />}
      {tab === "prompts" && <ProjectPromptsTab project={project.data} />}
      {tab === "save-points" && <SavePointsTab project={project.data} />}
      {tab === "portfolio" && <PortfolioTab project={project.data} />}
    </div>
  );
}
