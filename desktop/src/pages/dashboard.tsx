import { Sunrise, Sunset } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { ContinueRow } from "@/features/dashboard/continue-row";
import { FirstRunChecklist } from "@/features/dashboard/first-run-checklist";
import { formatToday, greetingFor } from "@/features/dashboard/greeting";
import { Hero } from "@/features/dashboard/hero";
import { RecommendationsCard } from "@/features/dashboard/recommendations-card";
import { ThisWeekCard } from "@/features/dashboard/this-week-card";
import { TimelineCard } from "@/features/dashboard/timeline-card";
import { TodayCard } from "@/features/dashboard/today-card";
import { IdleCard } from "@/features/reminders/idle-card";
import { toDateInput } from "@/lib/dates";
import { useOverlays } from "@/stores/overlays";

const LINK = "text-sm text-muted-foreground hover:text-foreground";

export function DashboardPage() {
  const [now] = useState(() => Date.now());
  const openDayFlow = useOverlays((state) => state.openDayFlow);

  return (
    <div className="flex flex-col gap-8">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="flex flex-col gap-1">
          <p className="text-label uppercase tracking-wider text-muted-foreground">
            {formatToday(now)}
          </p>
          <h1 className="text-display">{greetingFor(new Date(now).getHours())}</h1>
        </div>
        <div className="flex flex-col items-end gap-3">
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => openDayFlow("plan")}>
              <Sunrise size={16} />
              Plan my day
            </Button>
            <Button variant="outline" onClick={() => openDayFlow("review")}>
              <Sunset size={16} />
              Review my day
            </Button>
          </div>
          <nav className="flex gap-4" aria-label="Planning">
            <Link to="/planner" className={LINK}>
              Planner
            </Link>
            <Link to={`/notes?day=${toDateInput(now)}`} className={LINK}>
              Today's note
            </Link>
            <Link to="/review" className={LINK}>
              Weekly review
            </Link>
            <Link to="/recap" className={LINK}>
              Yearly recap
            </Link>
          </nav>
        </div>
      </header>
      <FirstRunChecklist />
      <Hero now={now} />
      <IdleCard />
      <div className="grid grid-cols-[repeat(auto-fit,minmax(22rem,1fr))] items-start gap-6">
        <TodayCard now={now} />
        <ThisWeekCard now={now} />
      </div>
      <ContinueRow />
      <RecommendationsCard />
      <TimelineCard now={now} />
    </div>
  );
}
