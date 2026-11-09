import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createTimeEntry,
  deleteTimeEntry,
  listTimeEntries,
  updateTimeEntry,
  type TimeEntryInput,
} from "@/lib/api/time-entries";

export function useTimeEntries(projectId: string) {
  return useQuery({
    queryKey: ["time-entries", projectId],
    queryFn: () => listTimeEntries(projectId),
  });
}

function useInvalidateTimeEntries(projectId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["time-entries", projectId] });
}

export function useCreateTimeEntry(projectId: string) {
  const invalidate = useInvalidateTimeEntries(projectId);
  return useMutation({
    mutationFn: (input: TimeEntryInput) => createTimeEntry(projectId, input),
    onSuccess: invalidate,
  });
}

export function useUpdateTimeEntry(projectId: string) {
  const invalidate = useInvalidateTimeEntries(projectId);
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: TimeEntryInput }) =>
      updateTimeEntry(id, input),
    onSuccess: invalidate,
  });
}

export function useDeleteTimeEntry(projectId: string) {
  const invalidate = useInvalidateTimeEntries(projectId);
  return useMutation({ mutationFn: deleteTimeEntry, onSuccess: invalidate });
}
