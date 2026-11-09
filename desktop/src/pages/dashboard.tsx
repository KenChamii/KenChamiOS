import { useState } from "react";
import { Link } from "react-router-dom";
import { ContinueRow } from "@/features/dashboard/continue-row";
import { StatsCard } from "@/features/dashboard/stats-card";
import { TimelineCard } from "@/features/dashboard/timeline-card";
import { TodayCard } from "@/features/dashboard/today-card";
import { WeeklyRecapCard } from "@/features/dashboard/weekly-recap-card";

export function DashboardPage() {
  const [now] = useState(() => Date.now());

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Dashboard</h1>
        <Link to="/recap" className="text-sm underline">
          Yearly recap
        </Link>
      </div>
      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
        <TodayCard now={now} />
        <ContinueRow />
      </div>
      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
        <WeeklyRecapCard now={now} />
        <StatsCard now={now} />
      </div>
      <TimelineCard now={now} />
    </div>
  );
}
