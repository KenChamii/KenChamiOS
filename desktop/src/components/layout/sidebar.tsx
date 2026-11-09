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
    <aside className="flex w-56 flex-col justify-between border-r border-border bg-card p-3">
      <nav className="flex flex-col gap-1">
        <span className="px-3 py-2 text-sm font-semibold">KenChamiOs</span>
        {links.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-muted",
                isActive && "bg-muted font-medium text-primary",
              )
            }
          >
            <Icon size={18} />
            {label}
          </NavLink>
        ))}
      </nav>
      <ThemeToggle />
    </aside>
  );
}
