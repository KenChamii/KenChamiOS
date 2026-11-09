import type { SelectHTMLAttributes } from "react";
import { cn } from "@/lib/cn";

export function Select({ className, ...props }: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={cn(
        "h-9 rounded-md border border-border bg-card px-2 text-sm focus-visible:outline-2 focus-visible:outline-primary disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}
