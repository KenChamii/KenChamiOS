import { Check, Flame, Repeat, X } from "lucide-react";
import { useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { AddItemForm } from "@/features/tasks/add-item-form";
import { useCreateHabit, useDeleteHabit, useHabits, useToggleHabit } from "@/hooks/use-habits";
import type { Habit } from "@/lib/api/habits";
import { cn } from "@/lib/cn";
import { toDateInput } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { stagger } from "@/lib/motion";

export function HabitsPage() {
  // the local day when the page opened; check-ins always go to this day
  const [today] = useState(() => toDateInput(Date.now()));
  const habits = useHabits(today);
  const create = useCreateHabit();
  const error = habits.error ?? create.error;

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Habits"
        subtitle="Check in once a day and keep the streak going"
        icon={Repeat}
        area="focus"
      />
      <div className="max-w-md">
        <AddItemForm
          placeholder="Add a habit, like Read 20 minutes"
          pending={create.isPending}
          onAdd={(name) => create.mutate(name)}
        />
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {habits.data && habits.data.length === 0 ? (
        <EmptyState icon={Repeat} area="focus" line="No habits yet. Add one above." />
      ) : (
        <ul className="flex flex-col divide-y divide-border rounded-2xl bg-card shadow-soft">
          {habits.data?.map((habit, index) => (
            <HabitRow key={habit.id} habit={habit} index={index} today={today} />
          ))}
        </ul>
      )}
    </div>
  );
}

function HabitRow({ habit, index, today }: { habit: Habit; index: number; today: string }) {
  const toggle = useToggleHabit();
  const remove = useDeleteHabit();
  const error = toggle.error ?? remove.error;

  return (
    <li className="stagger-in flex flex-col gap-1 px-4 py-3" style={stagger(index)}>
      <div className="flex items-center gap-4">
        <Button
          size="icon"
          variant={habit.checked_today ? "primary" : "outline"}
          aria-pressed={habit.checked_today}
          aria-label={`${habit.checked_today ? "Undo today's check-in for" : "Check in"} ${habit.name}`}
          disabled={toggle.isPending}
          onClick={() => toggle.mutate({ id: habit.id, day: today })}
        >
          <Check size={18} />
        </Button>
        <span className="flex-1 text-sm font-medium">{habit.name}</span>
        <span className="flex gap-1" aria-label="Last seven days">
          {habit.recent.map((checked, day) => (
            <span
              key={day}
              className={cn("h-2.5 w-2.5 rounded-full", checked ? "bg-focus" : "bg-muted")}
            />
          ))}
        </span>
        <span className="flex w-20 items-center justify-end gap-1 text-sm tabular-nums">
          <Flame size={16} className={habit.streak > 0 ? "text-focus" : "text-muted-foreground"} />
          {habit.streak} {habit.streak === 1 ? "day" : "days"}
        </span>
        <Button
          variant="ghost"
          size="icon"
          aria-label={`Delete ${habit.name}`}
          onClick={() => remove.mutate(habit.id)}
        >
          <X size={14} />
        </Button>
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </li>
  );
}
