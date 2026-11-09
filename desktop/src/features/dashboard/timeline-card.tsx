import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Tabs } from "@/components/ui/tabs";
import { useTimeline } from "@/hooks/use-dashboard";
import type { TimelineGroup } from "@/lib/api/dashboard";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";

const PAGE = 25;

type Filter = TimelineGroup | "all";

const FILTERS: { id: Filter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "projects", label: "Projects" },
  { id: "focus", label: "Focus" },
  { id: "media", label: "Media" },
];

export function TimelineCard({ now }: { now: number }) {
  const [filter, setFilter] = useState<Filter>("all");
  const [limit, setLimit] = useState(PAGE);
  const timeline = useTimeline(filter === "all" ? null : filter, limit);
  const items = timeline.data ?? [];

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Timeline</h2>
      <Tabs
        tabs={FILTERS}
        value={filter}
        onChange={(next) => {
          setFilter(next);
          setLimit(PAGE);
        }}
      />
      {timeline.error && <p className="text-sm text-destructive">{errorMessage(timeline.error)}</p>}
      {timeline.data?.length === 0 && (
        <p className="text-sm text-muted-foreground">Nothing has happened here yet.</p>
      )}
      <ul className="flex flex-col divide-y divide-border">
        {items.map((item) => (
          <li key={item.id} className="flex justify-between gap-4 py-2 text-sm">
            <span className="min-w-0 truncate">{item.summary}</span>
            <span className="shrink-0 text-xs text-muted-foreground">{timeAgo(item.at, now)}</span>
          </li>
        ))}
      </ul>
      {items.length === limit && (
        <div>
          <Button variant="outline" size="sm" onClick={() => setLimit(limit + PAGE)}>
            Show more
          </Button>
        </div>
      )}
    </Card>
  );
}
