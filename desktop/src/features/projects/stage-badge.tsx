import { Badge } from "@/components/ui/badge";
import type { Stage } from "@/lib/api/projects";
import { STAGE_ICONS, STAGE_LABELS } from "./meta";

export function StageBadge({ stage }: { stage: Stage }) {
  const Icon = STAGE_ICONS[stage];
  return (
    <Badge className="gap-1 bg-projects-soft text-projects">
      <Icon size={12} aria-hidden />
      {STAGE_LABELS[stage]}
    </Badge>
  );
}
