import { ChevronLeft, ChevronRight } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { fromDateInput } from "@/lib/dates";
import { addDays, isDay, mondayOf, type Day } from "@/lib/days";

/** The Monday of the week in `?week=`, or of this week. */
export function useWeekParam(today: Day): [Day, (monday: Day) => void] {
  const [params, setParams] = useSearchParams();
  const requested = params.get("week");
  const monday = mondayOf(requested && isDay(requested) ? requested : today);
  return [monday, (next) => setParams({ week: next })];
}

function label(day: Day): string {
  return new Date(fromDateInput(day) ?? 0).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
  });
}

type WeekNavProps = { monday: Day; today: Day; onChange: (monday: Day) => void };

export function WeekNav({ monday, today, onChange }: WeekNavProps) {
  const thisWeek = mondayOf(today);
  return (
    <div className="flex items-center gap-2">
      <Button
        variant="ghost"
        size="icon"
        aria-label="Previous week"
        onClick={() => onChange(addDays(monday, -7))}
      >
        <ChevronLeft size={16} />
      </Button>
      <span className="w-36 text-center text-sm font-medium tabular-nums">
        {label(monday)} – {label(addDays(monday, 6))}
      </span>
      <Button
        variant="ghost"
        size="icon"
        aria-label="Next week"
        onClick={() => onChange(addDays(monday, 7))}
      >
        <ChevronRight size={16} />
      </Button>
      {monday !== thisWeek && (
        <Button variant="outline" size="sm" onClick={() => onChange(thisWeek)}>
          This week
        </Button>
      )}
    </div>
  );
}
