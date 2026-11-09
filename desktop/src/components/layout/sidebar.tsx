import { Search } from "lucide-react";
import { NavLink } from "react-router-dom";
import { FocusPill } from "@/features/focus/focus-pill";
import { AREA_FILL, AREA_TEXT } from "@/lib/areas";
import { cn } from "@/lib/cn";
import { useOverlays } from "@/stores/overlays";
import { NAV_PAGES } from "./nav-pages";
import { ThemeToggle } from "./theme-toggle";

export function Sidebar() {
  const openPalette = useOverlays((state) => state.openPalette);
  return (
    <aside className="flex w-56 flex-col justify-between p-4">
      <nav className="flex flex-col gap-1">
        <span className="px-3 pb-4 pt-2 text-lg font-bold tracking-tight">KenChamiOs</span>
        <button
          type="button"
          onClick={() => openPalette()}
          className="mb-3 flex cursor-pointer items-center gap-3 rounded-xl bg-muted px-3 py-2 text-sm text-muted-foreground transition-colors duration-(--duration-fast) hover:text-foreground"
        >
          <Search size={16} />
          Search
          <kbd className="ml-auto font-sans text-xs">Ctrl K</kbd>
        </button>
        {NAV_PAGES.map(({ to, label, icon: Icon, area }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              cn(
                "relative flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-muted-foreground transition-[background-color,color,box-shadow] duration-(--duration-fast) ease-(--ease-out) hover:text-foreground",
                isActive && "bg-card font-medium text-foreground shadow-soft",
              )
            }
          >
            {({ isActive }) => (
              <>
                <span
                  className={cn(
                    "absolute -left-1 h-4 w-1 origin-center scale-y-0 rounded-full transition-transform duration-(--duration-base) ease-(--ease-out)",
                    area ? AREA_FILL[area] : "bg-foreground",
                    isActive && "scale-y-100",
                  )}
                />
                <Icon size={18} className={cn(area ? AREA_TEXT[area] : "text-foreground")} />
                {label}
              </>
            )}
          </NavLink>
        ))}
      </nav>
      <div className="flex flex-col gap-3">
        <FocusPill />
        <ThemeToggle />
      </div>
    </aside>
  );
}
