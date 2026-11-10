import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getAvailableMinutes,
  listCandidates,
  listCompleted,
  listFocusBlocks,
  listPlanned,
  setAvailableMinutes,
  setTaskPlan,
  type TaskPlan,
} from "@/lib/api/planner";
import type { Day } from "@/lib/days";

export function usePlanned(from: Day, to: Day) {
  return useQuery({
    queryKey: ["planner", "planned", from, to],
    queryFn: () => listPlanned(from, to),
  });
}

export function useCandidates(today: Day, until: number, query: string) {
  const search = query.trim() || null;
  return useQuery({
    queryKey: ["planner", "candidates", today, until, search],
    queryFn: () => listCandidates(today, until, search),
  });
}

export function useCompleted(from: number, to: number) {
  return useQuery({
    queryKey: ["planner", "completed", from, to],
    queryFn: () => listCompleted(from, to),
  });
}

export function useFocusBlocks(from: number, to: number) {
  return useQuery({
    queryKey: ["planner", "focus", from, to],
    queryFn: () => listFocusBlocks(from, to),
  });
}

export function useAvailableMinutes() {
  return useQuery({ queryKey: ["planner", "available"], queryFn: getAvailableMinutes });
}

export function useSetAvailableMinutes() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setAvailableMinutes,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["planner", "available"] }),
  });
}

// a plan change shows on the planner, the project board and the dashboard
export function useSetPlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, plan }: { id: string; plan: TaskPlan }) => setTaskPlan(id, plan),
    onSuccess: (task) =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ["planner"] }),
        queryClient.invalidateQueries({ queryKey: ["tasks", task.project_id] }),
        queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
      ]),
  });
}
