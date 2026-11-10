import { NotebookPen } from "lucide-react";
import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { monthStart, monthWeeks } from "@/features/notes/calendar";
import { MonthCalendar } from "@/features/notes/month-calendar";
import { NoteEditor } from "@/features/notes/note-editor";
import { useNoteDays } from "@/hooks/use-daily-notes";
import { fromDateInput, toDateInput } from "@/lib/dates";
import { addMonths, isDay, type Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";

export function NotesPage() {
  const [today] = useState(() => toDateInput(Date.now()));
  const [params, setParams] = useSearchParams();
  const requested = params.get("day");
  const day = requested && isDay(requested) ? requested : today;
  // the month on screen follows the open day until the arrows move it
  const [browsed, setBrowsed] = useState<{ from: Day; month: Day } | null>(null);
  const month = browsed?.from === day ? browsed.month : monthStart(day);
  const weeks = monthWeeks(month);
  const noteDays = useNoteDays(weeks[0][0], weeks[weeks.length - 1][6]);

  function open(next: Day) {
    setParams({ day: next });
  }

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Notes"
        subtitle="One note a day, for plans, thoughts and what happened"
        icon={NotebookPen}
        area="focus"
      >
        {day !== today && (
          <Button variant="outline" onClick={() => open(today)}>
            Today
          </Button>
        )}
      </PageHeader>
      {noteDays.error && <p className="text-sm text-destructive">{errorMessage(noteDays.error)}</p>}
      <div className="grid grid-cols-[18rem_1fr] items-start gap-6">
        <MonthCalendar
          month={month}
          selected={day}
          today={today}
          noteDays={new Set(noteDays.data)}
          onSelect={open}
          onMonth={(step) => setBrowsed({ from: day, month: addMonths(month, step) })}
        />
        <div className="flex min-w-0 flex-col gap-3">
          <h2 className="font-semibold tracking-tight">
            {new Date(fromDateInput(day) ?? 0).toLocaleDateString(undefined, {
              weekday: "long",
              day: "numeric",
              month: "long",
              year: "numeric",
            })}
          </h2>
          <NoteEditor key={day} day={day} />
        </div>
      </div>
    </div>
  );
}
