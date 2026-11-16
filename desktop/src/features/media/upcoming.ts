import type { UpcomingRelease } from "@/lib/api/media";

const DAY_MS = 24 * 60 * 60 * 1000;

export type UpcomingGroup = { label: string; releases: UpcomingRelease[] };

function startOfDay(ms: number): number {
  const date = new Date(ms);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/** Releases under Today, Tomorrow, This week and Later, in the order given. */
export function groupUpcoming(releases: UpcomingRelease[], nowMs: number): UpcomingGroup[] {
  const today = startOfDay(nowMs);
  const buckets: UpcomingGroup[] = [
    { label: "Today", releases: [] },
    { label: "Tomorrow", releases: [] },
    { label: "This week", releases: [] },
    { label: "Later", releases: [] },
  ];
  for (const release of releases) {
    const days = Math.floor((startOfDay(release.air_at_ms) - today) / DAY_MS);
    const index = days <= 0 ? 0 : days === 1 ? 1 : days < 7 ? 2 : 3;
    buckets[index].releases.push(release);
  }
  return buckets.filter((bucket) => bucket.releases.length > 0);
}
