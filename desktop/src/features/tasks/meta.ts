import type { SelectOption } from "@/components/ui/select";
import type { Priority, TaskStatus } from "@/lib/api/tasks";
import { ruleLabel } from "@/lib/recurrence";

export const STATUS_LABELS: Record<TaskStatus, string> = {
  todo: "To do",
  doing: "Doing",
  review: "Review",
  done: "Done",
};

export const PRIORITY_LABELS: Record<Priority, string> = {
  low: "Low",
  medium: "Medium",
  high: "High",
};

const COMMON_REPEATS = ["daily:1", "weekdays:1", "weekly:1", "weekly:2", "monthly:1"];

/** "Never" and the common repeats, plus the task's own rule when it is another one. */
export function repeatOptions(current: string | null): SelectOption<string>[] {
  const rules =
    current && !COMMON_REPEATS.includes(current) ? [...COMMON_REPEATS, current] : COMMON_REPEATS;
  return [
    { value: "", label: "Never" },
    ...rules.map((rule) => ({ value: rule, label: ruleLabel(rule) })),
  ];
}
