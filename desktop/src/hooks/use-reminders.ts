import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getIdleDays, listIdleProjects, setIdleDays, snoozeProject } from "@/lib/api/reminders";

export function useIdleProjects() {
  return useQuery({ queryKey: ["reminders", "idle"], queryFn: listIdleProjects });
}

export function useSnoozeProject() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: snoozeProject,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["reminders", "idle"] }),
  });
}

export function useIdleDays() {
  return useQuery({ queryKey: ["reminders", "idle-days"], queryFn: getIdleDays });
}

export function useSetIdleDays() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setIdleDays,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["reminders"] }),
  });
}
