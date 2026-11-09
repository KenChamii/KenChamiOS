import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import type { Area } from "@/lib/areas";
import { IconChip } from "./icon-chip";

type EmptyStateProps = { icon: LucideIcon; area?: Area; line: string; action?: ReactNode };

export function EmptyState({ icon, area, line, action }: EmptyStateProps) {
  return (
    <div className="flex animate-fade-in flex-col items-center gap-3 py-12 text-center">
      <IconChip icon={icon} area={area} size="lg" />
      <p className="text-body text-muted-foreground">{line}</p>
      {action}
    </div>
  );
}
