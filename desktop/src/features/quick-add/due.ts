import { fromDateInput } from "@/lib/dates";

export type DueChoice = "none" | "today" | "tomorrow" | "pick";

export const DUE_LABELS: Record<DueChoice, string> = {
  none: "No date",
  today: "Today",
  tomorrow: "Tomorrow",
  pick: "Pick date",
};

/** Local midnight of the chosen day, or null for no date or an empty pick. */
export function dueAt(choice: DueChoice, picked: string, now: number): number | null {
  const date = new Date(now);
  switch (choice) {
    case "none":
      return null;
    case "today":
      return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
    case "tomorrow":
      return new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1).getTime();
    case "pick":
      return fromDateInput(picked);
  }
}
