import { CalendarRange } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Stat } from "@/components/layout/stat";
import { Card } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { Select } from "@/components/ui/select";
import { reviewYears, yearReview } from "@/features/media/year-review";
import { useMedia } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

function monthName(index: number): string {
  return new Date(2000, index, 1).toLocaleString(undefined, { month: "short" });
}

export function MediaYearPage() {
  const media = useMedia();
  const entries = media.data ?? [];
  const years = reviewYears(entries);
  const [chosen, setChosen] = useState<number | null>(null);
  const year = chosen !== null && years.includes(chosen) ? chosen : years[0];

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Media year in review"
        subtitle="What you finished, by month"
        icon={CalendarRange}
        area="media"
      >
        {years.length > 0 && year !== undefined && (
          <Select
            aria-label="Year"
            className="w-28"
            value={String(year)}
            options={years.map((value) => ({ value: String(value), label: String(value) }))}
            onChange={(value) => setChosen(Number(value))}
          />
        )}
        <Link to="/media" className="text-sm text-muted-foreground hover:text-foreground">
          Back to the library
        </Link>
      </PageHeader>
      {media.error && <p className="text-sm text-destructive">{errorMessage(media.error)}</p>}
      {media.data && year === undefined ? (
        <EmptyState icon={CalendarRange} area="media" line="Finish a title and it shows up here." />
      ) : (
        year !== undefined && <YearBody entries={entries} year={year} />
      )}
    </div>
  );
}

function YearBody({ entries, year }: { entries: Parameters<typeof yearReview>[0]; year: number }) {
  const review = yearReview(entries, year);
  const busiest = Math.max(1, ...review.byMonth);
  return (
    <>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <Stat label="Finished" value={String(review.finished)} />
        <Stat label="Game hours" value={String(review.gameHours)} />
        <Stat label="Episodes" value={String(review.episodes)} />
        <Stat label="Chapters" value={String(review.chapters)} />
      </div>
      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold tracking-tight">By month</h2>
        <ul className="grid grid-cols-12 items-end gap-2" style={{ height: "7rem" }}>
          {review.byMonth.map((count, index) => (
            <li key={index} className="flex h-full flex-col items-center justify-end gap-1">
              <span className="text-xs tabular-nums text-muted-foreground">{count || ""}</span>
              <span
                className="w-full rounded-md bg-media"
                style={{ height: `${(count / busiest) * 100}%`, minHeight: count ? 4 : 0 }}
              />
              <span className="text-xs text-muted-foreground">{monthName(index)}</span>
            </li>
          ))}
        </ul>
      </Card>
      <div className="grid gap-4 sm:grid-cols-2">
        <Card className="flex flex-col gap-2">
          <h2 className="font-semibold tracking-tight">Top genres</h2>
          {review.topGenres.length === 0 ? (
            <p className="text-sm text-muted-foreground">No genres on these titles.</p>
          ) : (
            <ol className="flex flex-col gap-1 text-sm">
              {review.topGenres.map((item) => (
                <li key={item.genre} className="flex justify-between">
                  <span>{item.genre}</span>
                  <span className="tabular-nums text-muted-foreground">{item.count}</span>
                </li>
              ))}
            </ol>
          )}
        </Card>
        <Card className="flex flex-col gap-2">
          <h2 className="font-semibold tracking-tight">Highest rated</h2>
          {review.highestRated.length === 0 ? (
            <p className="text-sm text-muted-foreground">Rate a finished title to see it here.</p>
          ) : (
            <ol className="flex flex-col gap-1 text-sm">
              {review.highestRated.map((entry) => (
                <li key={entry.id} className="flex justify-between gap-3">
                  <Link to={`/media?open=${entry.id}`} className="truncate hover:underline">
                    {entry.title}
                  </Link>
                  <span className="tabular-nums text-muted-foreground">{entry.rating}/10</span>
                </li>
              ))}
            </ol>
          )}
        </Card>
      </div>
    </>
  );
}
