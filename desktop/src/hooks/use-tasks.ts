import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createTask,
  deleteTask,
  listTasks,
  moveTask,
  moveTaskToProject,
  updateTask,
  type Task,
  type TaskCreate,
  type TaskStatus,
  type TaskUpdate,
} from "@/lib/api/tasks";
import { restoreFromTrash } from "@/lib/api/trash";
import { nextDueAt } from "@/lib/recurrence";
import { useToasts } from "@/stores/toasts";

export function useTasks(projectId: string) {
  return useQuery({ queryKey: ["tasks", projectId], queryFn: () => listTasks(projectId) });
}

// tasks feed the project progress, last activity, the dashboard and the planner too
function useInvalidateTasks(projectId: string) {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
      queryClient.invalidateQueries({ queryKey: ["projects"] }),
      queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
      queryClient.invalidateQueries({ queryKey: ["planner"] }),
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
  return useMutation({
    mutationFn: (id: string) => deleteTask(id),
    onSuccess: (_, id) => {
      useToasts.getState().show("Task deleted", () => restoreFromTrash("task", id));
      return invalidate();
    },
  });
}

/** What a move needs to know about the task to repeat it and to undo a completion. */
export type MovableTask = Pick<Task, "id" | "status" | "recurrence" | "due_at">;

type Move = { task: MovableTask; status: TaskStatus; beforeId: string | null };

function completes(move: Move): boolean {
  return move.status === "done" && move.task.status !== "done";
}

export function useMoveTask(projectId: string) {
  const invalidate = useInvalidateTasks(projectId);
  return useMutation({
    mutationFn: (move: Move) =>
      moveTask(
        move.task.id,
        move.status,
        move.beforeId,
        completes(move) ? nextDueAt(move.task, Date.now()) : null,
      ),
    onSuccess: (moved, move) => {
      if (completes(move)) {
        const message = moved.next ? "Task completed, the next one is added" : "Task completed";
        useToasts.getState().show(message, async () => {
          await moveTask(move.task.id, move.task.status, null, null);
          if (moved.next) await deleteTask(moved.next.id);
        });
      }
      return invalidate();
    },
  });
}

export function useMoveTaskToProject(fromProjectId: string) {
  const queryClient = useQueryClient();
  const invalidate = useInvalidateTasks(fromProjectId);
  return useMutation({
    mutationFn: ({ id, projectId }: { id: string; projectId: string }) =>
      moveTaskToProject(id, projectId),
    onSuccess: (_, { projectId }) =>
      Promise.all([
        invalidate(),
        queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
      ]),
  });
}
