import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

type StatProps = { label: string; value: ReactNode; hero?: boolean; className?: string };

// small muted label above a bold value
export function Stat({ label, value, hero, className }: StatProps) {
  return (
    <div className={cn("flex flex-col gap-1", className)}>
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className={cn("font-bold tabular-nums", hero ? "text-hero" : "text-stat")}>
        {value}
      </span>
    </div>
  );
}
