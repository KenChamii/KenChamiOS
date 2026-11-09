import { formatDuration } from "@/features/focus/format";
import type { ProjectFocus } from "@/lib/api/dashboard";

export function FocusProjects({ projects }: { projects: ProjectFocus[] }) {
  if (projects.length === 0) {
    return <p className="text-sm text-muted-foreground">No focus time in this period.</p>;
  }
  const peak = Math.max(...projects.map((project) => project.seconds), 1);

  return (
    <ul className="flex flex-col gap-2">
      {projects.map((project) => (
        <li key={project.project_id} className="flex flex-col gap-1 text-sm">
          <div className="flex justify-between gap-4">
            <span className="truncate">{project.name}</span>
            <span className="text-muted-foreground">{formatDuration(project.seconds)}</span>
          </div>
          <div className="h-1.5 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full bg-primary"
              style={{
                width: `${(project.seconds / peak) * 100}%`,
                backgroundColor: project.color ?? undefined,
              }}
            />
          </div>
        </li>
      ))}
    </ul>
  );
}
