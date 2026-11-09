import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { ProjectSummary } from "@/lib/api/projects";
import { timeAgo } from "@/lib/dates";
import { STAGE_LABELS } from "./meta";

export function ProjectCard({ project }: { project: ProjectSummary }) {
  return (
    <Link to={`/projects/${project.id}`} className="block">
      <Card
        className="flex h-full flex-col gap-3 border-t-4 transition-[transform,box-shadow] duration-(--duration-base) ease-(--ease-out) hover:-translate-y-0.5"
        style={{ borderTopColor: project.color ?? undefined }}
      >
        <div className="flex items-start justify-between gap-2">
          <h2 className="font-semibold tracking-tight">{project.name}</h2>
          <Badge>{STAGE_LABELS[project.stage]}</Badge>
        </div>
        {project.description && (
          <p className="line-clamp-2 text-sm text-muted-foreground">{project.description}</p>
        )}
        <div className="mt-auto flex flex-col gap-1">
          <Progress value={project.tasks_done} max={project.tasks_total} />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>
              {project.tasks_done}/{project.tasks_total} tasks
            </span>
            <span>{timeAgo(project.last_activity_at)}</span>
          </div>
        </div>
        {project.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {project.tags.map((tag) => (
              <Badge key={tag}>{tag}</Badge>
            ))}
          </div>
        )}
      </Card>
    </Link>
  );
}
