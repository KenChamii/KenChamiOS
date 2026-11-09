import {
  CircleCheck,
  ClipboardList,
  Hammer,
  Lightbulb,
  SearchCheck,
  type LucideIcon,
} from "lucide-react";
import type { Stage } from "@/lib/api/projects";

export const STAGE_LABELS: Record<Stage, string> = {
  idea: "Idea",
  plan: "Plan",
  build: "Build",
  review: "Review",
  done: "Done",
};

export const STAGE_ICONS: Record<Stage, LucideIcon> = {
  idea: Lightbulb,
  plan: ClipboardList,
  build: Hammer,
  review: SearchCheck,
  done: CircleCheck,
};

export const PROJECT_COLORS = [
  "#6366f1",
  "#0ea5e9",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#ec4899",
  "#8b5cf6",
  "#64748b",
];
