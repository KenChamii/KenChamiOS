import { useState } from "react";
import { Link } from "react-router-dom";
import { ContinueRow } from "@/features/dashboard/continue-row";
import { FirstRunChecklist } from "@/features/dashboard/first-run-checklist";
import { formatToday, greetingFor } from "@/features/dashboard/greeting";
import { Hero } from "@/features/dashboard/hero";
import { RecommendationsCard } from "@/features/dashboard/recommendations-card";
import { ThisWeekCard } from "@/features/dashboard/this-week-card";
import { TimelineCard } from "@/features/dashboard/timeline-card";
import { TodayCard } from "@/features/dashboard/today-card";
import { IdleCard } from "@/features/reminders/idle-card";

export function DashboardPage() {
  const [now] = useState(() => Date.now());

  return (
    <div className="flex flex-col gap-8">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="flex flex-col gap-1">
          <p className="text-label uppercase tracking-wider text-muted-foreground">
            {formatToday(now)}
          </p>
          <h1 className="text-display">{greetingFor(new Date(now).getHours())}</h1>
        </div>
        <Link to="/recap" className="text-sm text-muted-foreground hover:text-foreground">
          Yearly recap
        </Link>
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
