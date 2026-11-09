import { TYPE_LABELS, UNIT_NAMES } from "@/features/media/meta";
import type { MediaTypeStat } from "@/lib/api/dashboard";

export function MediaStats({ stats }: { stats: MediaTypeStat[] }) {
  if (stats.length === 0) {
    return <p className="text-sm text-muted-foreground">No media progress in this period.</p>;
  }

  return (
    <ul className="flex flex-col gap-1 text-sm">
      {stats.map((stat) => (
        <li key={stat.media_type} className="flex justify-between gap-4">
          <span>{TYPE_LABELS[stat.media_type]}</span>
          <span className="text-muted-foreground">
            {stat.units} {UNIT_NAMES[stat.media_type]}
            {stat.completed > 0 && `, ${stat.completed} finished`}
          </span>
        </li>
      ))}
    </ul>
  );
}
