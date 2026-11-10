import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/cn";
import type { Day } from "@/lib/days";
import { monthWeeks } from "./calendar";

const WEEKDAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

type MonthCalendarProps = {
  month: Day;
  selected: Day;
  today: Day;
  noteDays: Set<Day>;
  onSelect: (day: Day) => void;
  onMonth: (step: number) => void;
};

export function MonthCalendar({
  month,
  selected,
  today,
  noteDays,
  onSelect,
  onMonth,
}: MonthCalendarProps) {
  const [year, monthNumber] = month.split("-").map(Number);
  const title = new Date(year, monthNumber - 1, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });

  return (
    <Card className="flex flex-col gap-3 p-4">
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="icon" aria-label="Previous month" onClick={() => onMonth(-1)}>
          <ChevronLeft size={16} />
        </Button>
        <span className="text-sm font-medium">{title}</span>
        <Button variant="ghost" size="icon" aria-label="Next month" onClick={() => onMonth(1)}>
          <ChevronRight size={16} />
        </Button>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center text-xs">
        {WEEKDAYS.map((name) => (
          <span key={name} className="text-muted-foreground">
            {name}
          </span>
        ))}
        {monthWeeks(month)
          .flat()
          .map((day) => (
            <button
              key={day}
              type="button"
              aria-label={day}
              aria-current={day === selected ? "date" : undefined}
              onClick={() => onSelect(day)}
              className={cn(
                "flex h-9 cursor-pointer flex-col items-center justify-center rounded-lg text-sm transition-colors duration-(--duration-fast) hover:bg-muted",
                day.slice(0, 7) !== month.slice(0, 7) && "text-muted-foreground",
                day === today && "font-semibold text-focus",
                day === selected && "bg-primary text-primary-foreground hover:bg-primary",
              )}
            >
              {Number(day.slice(8))}
              <span
                className={cn(
                  "h-1 w-1 rounded-full",
                  noteDays.has(day) && (day === selected ? "bg-primary-foreground" : "bg-focus"),
                )}
              />
            </button>
          ))}
      </div>
    </Card>
  );
}
