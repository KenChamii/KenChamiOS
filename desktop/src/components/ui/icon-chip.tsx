import type { LucideIcon } from "lucide-react";
import { AREA_SOFT, type Area } from "@/lib/areas";
import { cn } from "@/lib/cn";

type Size = "md" | "lg";

const SIZES: Record<Size, { box: string; icon: number }> = {
  md: { box: "h-9 w-9 rounded-xl", icon: 18 },
  lg: { box: "h-12 w-12 rounded-2xl", icon: 22 },
};

type IconChipProps = { icon: LucideIcon; area?: Area; size?: Size };

export function IconChip({ icon: Icon, area, size = "md" }: IconChipProps) {
  return (
    <span
      aria-hidden
      className={cn(
        "inline-flex shrink-0 items-center justify-center",
        SIZES[size].box,
        area ? AREA_SOFT[area] : "bg-muted text-foreground",
      )}
    >
      <Icon size={SIZES[size].icon} />
    </span>
  );
}
