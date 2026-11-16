import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createJob,
  deleteJob,
  getJobStats,
  listJobs,
  listJobsNeedingAttention,
  moveJob,
  updateJob,
  type JobInput,
  type JobStage,
} from "@/lib/api/jobs";
import { restoreFromTrash } from "@/lib/api/trash";
import { toDateInput } from "@/lib/dates";
import { useToasts } from "@/stores/toasts";

export function useJobs() {
  return useQuery({ queryKey: ["jobs", "list"], queryFn: listJobs });
}

export function useJobStats() {
  return useQuery({ queryKey: ["jobs", "stats"], queryFn: getJobStats });
}

export function useJobsNeedingAttention() {
  return useQuery({ queryKey: ["jobs", "attention"], queryFn: listJobsNeedingAttention });
}

function useRefreshJobs() {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["jobs"] }),
      queryClient.invalidateQueries({ queryKey: ["trash"] }),
    ]);
}

export function useCreateJob() {
  const refresh = useRefreshJobs();
  return useMutation({ mutationFn: (input: JobInput) => createJob(input), onSuccess: refresh });
}

export function useUpdateJob() {
  const refresh = useRefreshJobs();
  return useMutation({
    mutationFn: (input: { id: string; fields: JobInput }) => updateJob(input.id, input.fields),
    onSuccess: refresh,
  });
}

export function useMoveJob() {
  const refresh = useRefreshJobs();
  return useMutation({
    mutationFn: (input: { id: string; stage: JobStage }) =>
      moveJob(input.id, input.stage, toDateInput(Date.now())),
    onSuccess: refresh,
  });
}

export function useDeleteJob() {
  const refresh = useRefreshJobs();
  return useMutation({
    mutationFn: (id: string) => deleteJob(id),
    onSuccess: (_, id) => {
      void refresh();
      useToasts.getState().show("Application deleted", async () => {
        await restoreFromTrash("job", id);
        await refresh();
      });
    },
  });
}
