export const PROJECT_TABS = [
  "overview",
  "idea",
  "plan",
  "tasks",
  "time",
  "prompts",
  "save-points",
  "portfolio",
] as const;

export type ProjectTab = (typeof PROJECT_TABS)[number];

export type TabGroup = { id: string; label: string; tabs: { id: ProjectTab; label: string }[] };

export const TAB_GROUPS: TabGroup[] = [
  { id: "overview", label: "Overview", tabs: [{ id: "overview", label: "Overview" }] },
  {
    id: "plan",
    label: "Plan",
    tabs: [
      { id: "idea", label: "Idea" },
      { id: "plan", label: "Plan" },
    ],
  },
  {
    id: "work",
    label: "Work",
    tabs: [
      { id: "tasks", label: "Tasks" },
      { id: "time", label: "Time" },
      { id: "prompts", label: "Prompts" },
    ],
  },
  { id: "history", label: "History", tabs: [{ id: "save-points", label: "Save points" }] },
  { id: "share", label: "Share", tabs: [{ id: "portfolio", label: "Portfolio" }] },
];

/** The tab named in the url, or the overview for a missing or unknown one. */
export function parseTab(param: string | undefined): ProjectTab {
  return PROJECT_TABS.find((tab) => tab === param) ?? "overview";
}

export function groupOf(tab: ProjectTab): TabGroup {
  return TAB_GROUPS.find((group) => group.tabs.some((item) => item.id === tab)) ?? TAB_GROUPS[0];
}

export function projectPath(projectId: string, tab: ProjectTab = "overview"): string {
  return tab === "overview" ? `/projects/${projectId}` : `/projects/${projectId}/${tab}`;
}
