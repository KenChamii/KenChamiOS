// full class names, so tailwind can see them
export type Area = "projects" | "media" | "focus" | "workflow";

export const AREA_TEXT: Record<Area, string> = {
  projects: "text-projects",
  media: "text-media",
  focus: "text-focus",
  workflow: "text-workflow",
};

export const AREA_SOFT: Record<Area, string> = {
  projects: "bg-projects-soft text-projects",
  media: "bg-media-soft text-media",
  focus: "bg-focus-soft text-focus",
  workflow: "bg-workflow-soft text-workflow",
};

export const AREA_FILL: Record<Area, string> = {
  projects: "bg-projects",
  media: "bg-media",
  focus: "bg-focus",
  workflow: "bg-workflow",
};
