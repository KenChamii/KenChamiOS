import { CalendarCheck, Flame } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Stat } from "@/components/layout/stat";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { FocusProjects } from "@/features/dashboard/focus-projects";
import { weekRange } from "@/features/dashboard/ranges";
import { formatDuration } from "@/features/focus/format";
import { habitWeek, slipped } from "@/features/planner/review";
import { weekDays } from "@/features/planner/week-grid";
import { useWeekParam, WeekNav } from "@/features/planner/week-nav";
import { useRecap } from "@/hooks/use-dashboard";
import { useHabitChecks, useHabits } from "@/hooks/use-habits";
import { useCompleted, usePlanned } from "@/hooks/use-planner";
import type { PlannedTask } from "@/lib/api/planner";
import { fromDateInput, toDateInput } from "@/lib/dates";
import type { Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";

export function WeeklyReviewPage() {
  const [today] = useState(() => toDateInput(Date.now()));
  const [monday, setWeek] = useWeekParam(today);
  const week = weekDays(monday);
  const range = weekRange(fromDateInput(monday) ?? 0);
  const recap = useRecap(range);
  const completed = useCompleted(range.from, range.to);
  const planned = usePlanned(week[0], week[6]);
  const missed = slipped(planned.data ?? [], today);
  const error = recap.error ?? completed.error ?? planned.error;
  // a past week is written up on its last day, the current one today
  const noteDay = week[6] < today ? week[6] : today;

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Weekly review"
        subtitle="What got done, where the time went and what slipped"
        icon={CalendarCheck}
        area="focus"
      >
        <WeekNav monday={monday} today={today} onChange={setWeek} />
        <Link
          to={`/notes?day=${noteDay}`}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          Write a note
        </Link>
      </PageHeader>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <Card className="grid grid-cols-3 gap-6">
        <Stat label="Tasks done" value={completed.data?.length ?? 0} />
        <Stat label="Focused" value={formatDuration(recap.data?.focus_seconds ?? 0)} />
        <Stat label="Slipped" value={missed.length} />
      </Card>
      <div className="grid grid-cols-[repeat(auto-fit,minmax(22rem,1fr))] items-start gap-6">
        <TaskListCard
          title="Done"
          empty="Nothing completed this week."
          tasks={completed.data ?? []}
        />
        <Card className="flex flex-col gap-3">
          <h2 className="font-semibold tracking-tight">Focus per project</h2>
          <FocusProjects projects={recap.data?.focus_by_project ?? []} />
        </Card>
        <TaskListCard
          title="Slipped"
          empty="Everything planned for a past day got done."
          tasks={missed}
          showDay
        />
        <HabitsCard week={week} today={today} />
      </div>
    </div>
  );
}

type TaskListCardProps = { title: string; empty: string; tasks: PlannedTask[]; showDay?: boolean };

function TaskListCard({ title, empty, tasks, showDay }: TaskListCardProps) {
  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">{title}</h2>
      {tasks.length === 0 && <p className="text-sm text-muted-foreground">{empty}</p>}
      <ul className="flex flex-col gap-1.5 text-sm">
        {tasks.map((task) => (
          <li key={task.id} className="flex justify-between gap-3">
            <span className="truncate">
              {task.title}
              <span className="text-muted-foreground"> · {task.project_name}</span>
            </span>
            {showDay && task.planned_on && (
              <span className="shrink-0 text-muted-foreground">
                {new Date(fromDateInput(task.planned_on) ?? 0).toLocaleDateString(undefined, {
                  weekday: "short",
                })}
              </span>
            )}
          </li>
        ))}
      </ul>
    </Card>
  );
}

function HabitsCard({ week, today }: { week: Day[]; today: Day }) {
  const habits = useHabits(today);
  const checks = useHabitChecks(week[0], week[6]);
  const list = habits.data ?? [];
  const error = habits.error ?? checks.error;
  const rates = habitWeek(
    list.map((habit) => habit.id),
    checks.data ?? [],
    week,
    today,
  );

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Habits</h2>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {list.length === 0 && <p className="text-sm text-muted-foreground">No habits yet.</p>}
      <ul className="flex flex-col gap-3 text-sm">
        {list.map((habit, index) => (
          <li key={habit.id} className="flex flex-col gap-1">
            <div className="flex justify-between gap-3">
              <span className="truncate">{habit.name}</span>
              <span className="flex shrink-0 items-center gap-3 tabular-nums text-muted-foreground">
                {rates[index].checked}/{rates[index].days} days · {rates[index].percent}%
                <span className="flex items-center gap-1">
                  <Flame size={14} className={habit.streak > 0 ? "text-focus" : undefined} />
                  {habit.streak}
                </span>
              </span>
            </div>
            <Progress value={rates[index].checked} max={rates[index].days} area="focus" />
          </li>
        ))}
      </ul>
    </Card>
  );
}
