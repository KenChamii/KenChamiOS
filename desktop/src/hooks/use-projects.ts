import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createProject,
  deleteProject,
  getProject,
  listProjects,
  setProjectStage,
  updateProject,
  type ProjectFilter,
  type ProjectInput,
  type Stage,
} from "@/lib/api/projects";
import { restoreFromTrash } from "@/lib/api/trash";
import { useToasts } from "@/stores/toasts";

export function useProjects(filter: ProjectFilter) {
  return useQuery({ queryKey: ["projects", "list", filter], queryFn: () => listProjects(filter) });
}

export function useProject(id: string) {
  return useQuery({ queryKey: ["projects", "detail", id], queryFn: () => getProject(id) });
}

function useInvalidateProjects() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["projects"] });
}

export function useCreateProject() {
  const invalidate = useInvalidateProjects();
  return useMutation({ mutationFn: createProject, onSuccess: invalidate });
}

export function useUpdateProject(id: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (input: ProjectInput) => updateProject(id, input),
    onSuccess: invalidate,
  });
}

export function useDeleteProject(id: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: () => deleteProject(id),
    onSuccess: () => {
      useToasts.getState().show("Project deleted", () => restoreFromTrash("project", id));
      return invalidate();
    },
  });
}

export function useSetStage(id: string) {
  const invalidate = useInvalidateProjects();
  return useMutation({
    mutationFn: (stage: Stage) => setProjectStage(id, stage),
    onSuccess: invalidate,
  });
}
