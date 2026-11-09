import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createTask,
  deleteTask,
  listTasks,
  moveTask,
  updateTask,
  type TaskCreate,
  type TaskStatus,
  type TaskUpdate,
} from "@/lib/api/tasks";

export function useTasks(projectId: string) {
  return useQuery({ queryKey: ["tasks", projectId], queryFn: () => listTasks(projectId) });
}

// tasks feed the project progress and last activity too
function useInvalidateTasks(projectId: string) {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
      queryClient.invalidateQueries({ queryKey: ["projects"] }),
    ]);
}

export function useCreateTask(projectId: string) {
  const invalidate = useInvalidateTasks(projectId);
  return useMutation({
    mutationFn: (input: TaskCreate) => createTask(input),
    onSuccess: invalidate,
  });
}

export function useUpdateTask(projectId: string) {
  const invalidate = useInvalidateTasks(projectId);
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: TaskUpdate }) => updateTask(id, input),
    onSuccess: invalidate,
  });
}

export function useDeleteTask(projectId: string) {
  const invalidate = useInvalidateTasks(projectId);
  return useMutation({ mutationFn: (id: string) => deleteTask(id), onSuccess: invalidate });
}

export function useMoveTask(projectId: string) {
  const invalidate = useInvalidateTasks(projectId);
  return useMutation({
    mutationFn: (move: { id: string; status: TaskStatus; beforeId: string | null }) =>
      moveTask(move.id, move.status, move.beforeId),
    onSuccess: invalidate,
  });
}
