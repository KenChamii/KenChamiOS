import { Link } from "react-router-dom";
import type { ProjectSummary } from "@/lib/api/projects";
import { BriefCard } from "./brief-card";
import { PromptLibrary } from "./prompt-library";

export function ProjectPromptsTab({ project }: { project: ProjectSummary }) {
  return (
    <div className="flex flex-col gap-6">
      <BriefCard project={project} />
      <section className="flex flex-col gap-3">
        <div className="flex items-baseline justify-between gap-4">
          <h2 className="font-semibold tracking-tight">Prompts</h2>
          <Link to="/workflow" className="text-sm text-muted-foreground hover:text-foreground">
            Full library
          </Link>
        </div>
        <p className="text-sm text-muted-foreground">
          Copy a task as a prompt from its row in Tasks.
        </p>
        <PromptLibrary projectId={project.id} />
      </section>
    </div>
  );
}
