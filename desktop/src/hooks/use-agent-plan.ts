import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createPlanTasks, previewPlan, type PlanProposal } from "@/lib/api/agent";

export function usePlanPreview(projectId: string) {
  return useMutation({ mutationFn: () => previewPlan(projectId) });
}

export function useCreatePlanTasks(projectId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (proposal: PlanProposal) => createPlanTasks(projectId, proposal),
    // new tasks move progress, the dashboard and the planner too
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["projects"] }),
        queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
        queryClient.invalidateQueries({ queryKey: ["planner"] }),
      ]),
  });
}
