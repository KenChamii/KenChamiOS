import { Plus } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { ProjectCard } from "@/features/projects/project-card";
import { NewProjectDialog } from "@/features/projects/project-dialog";
import { useProjects } from "@/hooks/use-projects";
import { errorMessage } from "@/lib/errors";

export function ProjectsPage() {
  const navigate = useNavigate();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [tag, setTag] = useState("");
  const [showArchived, setShowArchived] = useState(false);

  const projects = useProjects({
    search: search || null,
    tag: tag || null,
    include_archived: showArchived,
  });
  // unfiltered list, so the tag menu keeps all tags while a filter is active
  const everything = useProjects({ search: null, tag: null, include_archived: showArchived });
  const allTags = [...new Set((everything.data ?? []).flatMap((project) => project.tags))].sort();
  const isFiltering = search !== "" || tag !== "";

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title="Projects" subtitle="Everything you are building">
        <Button onClick={() => setDialogOpen(true)}>
          <Plus size={16} />
          New project
        </Button>
      </PageHeader>

      <div className="flex flex-wrap items-center gap-3">
        <Input
          className="max-w-xs"
          placeholder="Search projects"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <Select value={tag} onChange={(e) => setTag(e.target.value)} aria-label="Filter by tag">
          <option value="">All tags</option>
          {allTags.map((value) => (
            <option key={value} value={value}>
              {value}
            </option>
          ))}
        </Select>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={showArchived}
            onChange={(e) => setShowArchived(e.target.checked)}
          />
          Show archived
        </label>
      </div>

      {projects.error ? (
        <p className="text-sm text-destructive">{errorMessage(projects.error)}</p>
      ) : projects.data?.length === 0 ? (
        <div className="flex flex-col items-center gap-3 py-16 text-center">
          <p className="text-muted-foreground">
            {isFiltering
              ? "No projects match these filters."
              : "No projects yet. Start with an idea."}
          </p>
          {!isFiltering && (
            <Button onClick={() => setDialogOpen(true)}>Create your first project</Button>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-6">
          {projects.data?.map((project) => (
            <ProjectCard key={project.id} project={project} />
          ))}
        </div>
      )}

      <NewProjectDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onCreated={(project) => navigate(`/projects/${project.id}`)}
      />
    </div>
  );
}
