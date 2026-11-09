import { useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { ContinueRow } from "@/features/dashboard/continue-row";
import { Hero } from "@/features/dashboard/hero";
import { RecommendationsCard } from "@/features/dashboard/recommendations-card";
import { IdleCard } from "@/features/reminders/idle-card";
import { StatsCard } from "@/features/dashboard/stats-card";
import { TimelineCard } from "@/features/dashboard/timeline-card";
import { TodayCard } from "@/features/dashboard/today-card";
import { WeeklyRecapCard } from "@/features/dashboard/weekly-recap-card";

export function DashboardPage() {
  const [now] = useState(() => Date.now());

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title="Dashboard" subtitle="Where things stand today">
        <Link to="/recap" className="text-sm text-muted-foreground hover:text-foreground">
          Yearly recap
        </Link>
      </PageHeader>
      <Hero now={now} />
      <IdleCard />
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <TodayCard now={now} />
        <ContinueRow />
      </div>
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <WeeklyRecapCard now={now} />
        <StatsCard now={now} />
      </div>
      <RecommendationsCard />
      <TimelineCard now={now} />
    </div>
  );
}
