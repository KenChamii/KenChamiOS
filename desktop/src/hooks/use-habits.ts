import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createHabit,
  deleteHabit,
  listHabitChecks,
  listHabits,
  toggleHabit,
} from "@/lib/api/habits";
import { restoreFromTrash } from "@/lib/api/trash";
import type { Day } from "@/lib/days";
import { useToasts } from "@/stores/toasts";

export function useHabits(today: Day) {
  return useQuery({ queryKey: ["habits", today], queryFn: () => listHabits(today) });
}

function useInvalidateHabits() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["habits"] });
}

export function useCreateHabit() {
  const invalidate = useInvalidateHabits();
  return useMutation({ mutationFn: createHabit, onSuccess: invalidate });
}

export function useDeleteHabit() {
  const invalidate = useInvalidateHabits();
  return useMutation({
    mutationFn: deleteHabit,
    onSuccess: (_, id) => {
      useToasts.getState().show("Habit deleted", () => restoreFromTrash("habit", id));
      return invalidate();
    },
  });
}

export function useToggleHabit() {
  const invalidate = useInvalidateHabits();
  return useMutation({
    mutationFn: ({ id, day }: { id: string; day: Day }) => toggleHabit(id, day),
    onSuccess: invalidate,
  });
}

export function useHabitChecks(from: Day, to: Day) {
  return useQuery({
    queryKey: ["habits", "checks", from, to],
    queryFn: () => listHabitChecks(from, to),
  });
}
