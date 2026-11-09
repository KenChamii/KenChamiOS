import { Film, FolderKanban, GitBranch, LayoutDashboard, Settings } from "lucide-react";
import { NavLink } from "react-router-dom";
import { cn } from "@/lib/cn";
import { ThemeToggle } from "./theme-toggle";

const links = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/projects", label: "Projects", icon: FolderKanban },
  { to: "/workflow", label: "Workflow", icon: GitBranch },
  { to: "/media", label: "Media", icon: Film },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  return (
    <aside className="flex w-52 flex-col justify-between p-4">
      <nav className="flex flex-col gap-1">
        <span className="px-3 pb-5 pt-2 text-lg font-bold tracking-tight">KenChamiOs</span>
        {links.map(({ to, label, icon: Icon }) => (
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
                    "absolute -left-1 h-4 w-1 origin-center scale-y-0 rounded-full bg-accent transition-transform duration-(--duration-base) ease-(--ease-out)",
                    isActive && "scale-y-100",
                  )}
                />
                <Icon size={18} />
                {label}
              </>
            )}
          </NavLink>
        ))}
      </nav>
      <ThemeToggle />
    </aside>
  );
}
