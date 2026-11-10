import { CalendarDays } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { WeekBoard } from "@/features/planner/week-board";
import { weekDays } from "@/features/planner/week-grid";
import { useWeekParam, WeekNav } from "@/features/planner/week-nav";
import { toDateInput } from "@/lib/dates";

export function PlannerPage() {
  const [today] = useState(() => toDateInput(Date.now()));
  const [monday, setWeek] = useWeekParam(today);

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Planner"
        subtitle="Put the week's tasks on days and hours"
        icon={CalendarDays}
        area="focus"
      >
        <WeekNav monday={monday} today={today} onChange={setWeek} />
        <Link
          to={`/review?week=${monday}`}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          Weekly review
        </Link>
      </PageHeader>
      <WeekBoard week={weekDays(monday)} today={today} />
    </div>
  );
}
