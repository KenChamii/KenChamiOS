import {
  Film,
  FolderKanban,
  GitBranch,
  Inbox,
  LayoutDashboard,
  Repeat,
  Settings,
  Trash2,
  type LucideIcon,
} from "lucide-react";
import type { Area } from "@/lib/areas";

export type NavPage = {
  to: string;
  label: string;
  icon: LucideIcon;
  area?: Area;
  /** pressed after G to jump here */
  key: string;
};

export const NAV_PAGES: NavPage[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard, key: "d" },
  { to: "/inbox", label: "Inbox", icon: Inbox, key: "i" },
  { to: "/projects", label: "Projects", icon: FolderKanban, area: "projects", key: "p" },
  { to: "/workflow", label: "Workflow", icon: GitBranch, area: "workflow", key: "w" },
  { to: "/media", label: "Media", icon: Film, area: "media", key: "m" },
  { to: "/habits", label: "Habits", icon: Repeat, area: "focus", key: "h" },
  { to: "/trash", label: "Trash", icon: Trash2, key: "t" },
  { to: "/settings", label: "Settings", icon: Settings, key: "s" },
];
