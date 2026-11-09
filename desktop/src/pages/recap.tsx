import { CalendarRange } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Stat } from "@/components/layout/stat";
import { Card } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { FocusProjects } from "@/features/dashboard/focus-projects";
import { MediaStats } from "@/features/dashboard/media-stats";
import { yearRange } from "@/features/dashboard/ranges";
import { DayChart } from "@/features/focus/day-chart";
import { formatDuration } from "@/features/focus/format";
import { useRecap } from "@/hooks/use-dashboard";
import { errorMessage } from "@/lib/errors";

const YEARS_BACK = 5;

export function RecapPage() {
  const [thisYear] = useState(() => new Date().getFullYear());
  const [year, setYear] = useState(thisYear);
  const range = yearRange(year);
  const recap = useRecap(range);
  const view = recap.data;
  const months = (view?.focus_by_bucket ?? []).map((seconds, index) => ({
    date: new Date(range.buckets[index]).toLocaleString(undefined, { month: "long" }),
    seconds,
  }));

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Yearly recap"
        subtitle="A look back at the year"
        icon={CalendarRange}
        area="focus"
      >
        <Select
          aria-label="Year"
          className="w-28"
          value={String(year)}
          options={Array.from({ length: YEARS_BACK + 1 }, (_, back) => {
            const value = String(thisYear - back);
            return { value, label: value };
          })}
          onChange={(value) => setYear(Number(value))}
        />
        <Link to="/" className="text-sm text-muted-foreground hover:text-foreground">
          Dashboard
        </Link>
      </PageHeader>
      {recap.error && <p className="text-sm text-destructive">{errorMessage(recap.error)}</p>}
      {view && (
        <>
          <Card className="p-8">
            <Stat hero label={`Focused in ${year}`} value={formatDuration(view.focus_seconds)} />
          </Card>
          <Card className="flex flex-col gap-3">
            <h2 className="font-semibold tracking-tight">Per month</h2>
            <DayChart days={months} />
          </Card>
          <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
            <Card className="flex flex-col gap-3">
              <h2 className="font-semibold tracking-tight">Focus per project</h2>
              <FocusProjects projects={view.focus_by_project} />
            </Card>
            <Card className="flex flex-col gap-3">
              <h2 className="font-semibold tracking-tight">Media</h2>
              <MediaStats stats={view.media} />
              {view.top_genres.length > 0 && (
                <p className="text-sm text-muted-foreground">
                  Top genres: {view.top_genres.map((g) => `${g.genre} (${g.count})`).join(", ")}
                </p>
              )}
            </Card>
          </div>
          <Card className="flex flex-col gap-1 text-sm">
            <h2 className="font-semibold tracking-tight">Projects</h2>
            <p>{view.stage_changes} stage moves.</p>
            <p>
              {view.projects_finished.length === 0
                ? "No project finished."
                : `Finished: ${view.projects_finished.join(", ")}.`}
            </p>
          </Card>
        </>
      )}
    </div>
  );
}
