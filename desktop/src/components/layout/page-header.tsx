import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { IconChip } from "@/components/ui/icon-chip";
import type { Area } from "@/lib/areas";

type PageHeaderProps = {
  title: string;
  subtitle?: string;
  icon?: LucideIcon;
  area?: Area;
  children?: ReactNode;
};

export function PageHeader({ title, subtitle, icon, area, children }: PageHeaderProps) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-4">
      <div className="flex items-center gap-3">
        {icon && <IconChip icon={icon} area={area} />}
        <div className="flex flex-col gap-0.5">
          <h1 className="text-title">{title}</h1>
          {subtitle && <p className="text-body text-muted-foreground">{subtitle}</p>}
        </div>
      </div>
      {children && <div className="flex items-center gap-2">{children}</div>}
    </header>
  );
}
