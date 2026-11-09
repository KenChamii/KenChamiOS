import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createSavePoint,
  deleteSavePoint,
  exportSavePoints,
  listSavePoints,
  type SavePointInput,
} from "@/lib/api/save-points";
import { restoreFromTrash } from "@/lib/api/trash";
import { useToasts } from "@/stores/toasts";

export function useSavePoints(projectId: string) {
  return useQuery({
    queryKey: ["save-points", projectId],
    queryFn: () => listSavePoints(projectId),
  });
}

// save points feed the project last activity too
function useInvalidateSavePoints(projectId: string) {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["save-points", projectId] }),
      queryClient.invalidateQueries({ queryKey: ["projects"] }),
    ]);
}

export function useCreateSavePoint(projectId: string) {
  const invalidate = useInvalidateSavePoints(projectId);
  return useMutation({
    mutationFn: (input: SavePointInput) => createSavePoint(projectId, input),
    onSuccess: invalidate,
  });
}

export function useDeleteSavePoint(projectId: string) {
  const invalidate = useInvalidateSavePoints(projectId);
  return useMutation({
    mutationFn: deleteSavePoint,
    onSuccess: (_, id) => {
      useToasts.getState().show("Save point deleted", () => restoreFromTrash("save_point", id));
      return invalidate();
    },
  });
}

export function useExportSavePoints(projectId: string) {
  return useMutation({ mutationFn: (path: string) => exportSavePoints(projectId, path) });
}
