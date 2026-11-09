import type { ProjectSummary } from "@/lib/api/projects";
import { AssetsCard } from "./assets-card";
import { ExportCard } from "./export-card";

export function PortfolioTab({ project }: { project: ProjectSummary }) {
  return (
    <div className="flex flex-col gap-4">
      <ExportCard project={project} />
      <AssetsCard projectId={project.id} />
    </div>
  );
}
