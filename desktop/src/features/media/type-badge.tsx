import { Badge } from "@/components/ui/badge";
import type { MediaType } from "@/lib/api/media";
import { TYPE_ICONS, TYPE_LABELS } from "./meta";

export function TypeBadge({ type }: { type: MediaType }) {
  const Icon = TYPE_ICONS[type];
  return (
    <Badge className="gap-1 bg-media-soft text-media">
      <Icon size={12} aria-hidden />
      {TYPE_LABELS[type]}
    </Badge>
  );
}
