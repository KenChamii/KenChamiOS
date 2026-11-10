import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { appendDailyNote, getDailyNote, listNoteDays, saveDailyNote } from "@/lib/api/daily-notes";
import type { Day } from "@/lib/days";

export function useDailyNote(day: Day) {
  return useQuery({ queryKey: ["daily-notes", "note", day], queryFn: () => getDailyNote(day) });
}

export function useNoteDays(from: Day, to: Day) {
  return useQuery({
    queryKey: ["daily-notes", "days", from, to],
    queryFn: () => listNoteDays(from, to),
  });
}

// the open editor keeps its own text, so only the calendar dots are refreshed
export function useSaveDailyNote() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ day, body }: { day: Day; body: string }) => saveDailyNote(day, body),
    onSuccess: (note) => {
      queryClient.setQueryData(["daily-notes", "note", note.day], note);
      return queryClient.invalidateQueries({ queryKey: ["daily-notes", "days"] });
    },
  });
}

export function useAppendDailyNote() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ day, text }: { day: Day; text: string }) => appendDailyNote(day, text),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["daily-notes"] }),
  });
}
