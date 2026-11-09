import { useQuery } from "@tanstack/react-query";
import {
  getRecap,
  listDueTasks,
  listTimeline,
  type RecapRange,
  type TimelineGroup,
} from "@/lib/api/dashboard";

export function useDueTasks(until: number) {
  return useQuery({ queryKey: ["dashboard", "due", until], queryFn: () => listDueTasks(until) });
}

export function useTimeline(group: TimelineGroup | null, limit: number) {
  return useQuery({
    queryKey: ["dashboard", "timeline", group, limit],
    queryFn: () => listTimeline(group, limit),
  });
}

export function useRecap(range: RecapRange) {
  return useQuery({
    queryKey: ["dashboard", "recap", range.from, range.to],
    queryFn: () => getRecap(range),
  });
}
