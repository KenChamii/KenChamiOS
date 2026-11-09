import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createChecklistItem,
  deleteChecklistItem,
  listChecklist,
  updateChecklistItem,
} from "@/lib/api/checklist";

export function useChecklist(taskId: string) {
  return useQuery({ queryKey: ["checklist", taskId], queryFn: () => listChecklist(taskId) });
}

function useInvalidateChecklist(taskId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["checklist", taskId] });
}

export function useAddChecklistItem(taskId: string) {
  const invalidate = useInvalidateChecklist(taskId);
  return useMutation({
    mutationFn: (text: string) => createChecklistItem(taskId, text),
    onSuccess: invalidate,
  });
}

export function useUpdateChecklistItem(taskId: string) {
  const invalidate = useInvalidateChecklist(taskId);
  return useMutation({
    mutationFn: (item: { id: string; text: string; done: boolean }) =>
      updateChecklistItem(item.id, item.text, item.done),
    onSuccess: invalidate,
  });
}

export function useDeleteChecklistItem(taskId: string) {
  const invalidate = useInvalidateChecklist(taskId);
  return useMutation({
    mutationFn: (id: string) => deleteChecklistItem(id),
    onSuccess: invalidate,
  });
}
