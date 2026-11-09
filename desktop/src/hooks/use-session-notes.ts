import { useMutation, useQueryClient } from "@tanstack/react-query";
import { confirmSessionNote, previewSessionNote } from "@/lib/api/session-notes";

export function usePreviewSessionNote(projectId: string) {
  return useMutation({ mutationFn: (text: string) => previewSessionNote(projectId, text) });
}

// a session note adds a save point and changes tasks, so refresh both
export function useConfirmSessionNote(projectId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: confirmSessionNote,
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ["save-points", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["projects"] }),
      ]),
  });
}
