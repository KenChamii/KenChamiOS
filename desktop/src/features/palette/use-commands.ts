import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { NAV_PAGES } from "@/components/layout/nav-pages";
import { endOfDay } from "@/features/dashboard/ranges";
import { TYPE_LABELS } from "@/features/media/meta";
import { useDueTasks } from "@/hooks/use-dashboard";
import { useMedia } from "@/hooks/use-media";
import { useProjects } from "@/hooks/use-projects";
import {
  usePauseTimer,
  useResumeTimer,
  useStartTimer,
  useStopTimer,
  useTimer,
} from "@/hooks/use-timer";
import { toDateInput } from "@/lib/dates";
import { useOverlays } from "@/stores/overlays";
import type { Command } from "./filter";

const ACTIVE_PROJECTS = { search: null, tag: null, include_archived: false };

/** Everything the palette can do, built from data the app already has loaded. */
export function useCommands(): Command[] {
  const navigate = useNavigate();
  const [now] = useState(() => Date.now());
  const projects = useProjects(ACTIVE_PROJECTS);
  const media = useMedia();
  const due = useDueTasks(endOfDay(now));
  const timer = useTimer();
  const start = useStartTimer();
  const pause = usePauseTimer();
  const resume = useResumeTimer();
  const stop = useStopTimer();
  const { openQuickAdd, setShortcuts, openDayFlow } = useOverlays();

  const pages: Command[] = [
    ...NAV_PAGES.map((page) => ({
      id: `page:${page.to}`,
      label: page.label,
      group: "Go to",
      run: () => navigate(page.to),
    })),
    { id: "page:/recap", label: "Yearly recap", group: "Go to", run: () => navigate("/recap") },
    {
      id: "page:/review",
      label: "Weekly review",
      group: "Go to",
      keywords: "week slipped done habits",
      run: () => navigate("/review"),
    },
    {
      id: "page:/notes/today",
      label: "Today's note",
      group: "Go to",
      keywords: "daily note journal",
      run: () => navigate(`/notes?day=${toDateInput(Date.now())}`),
    },
  ];

  const actions: Command[] = [
    {
      id: "action:plan-day",
      label: "Plan my day",
      group: "Plan",
      keywords: "morning today estimate",
      run: () => openDayFlow("plan"),
    },
    {
      id: "action:review-day",
      label: "Review my day",
      group: "Plan",
      keywords: "evening done tomorrow",
      run: () => openDayFlow("review"),
    },
    {
      id: "action:task",
      label: "Add task",
      group: "Create",
      keywords: "new todo",
      run: () => openQuickAdd("task"),
    },
    {
      id: "action:project",
      label: "New project",
      group: "Create",
      keywords: "add create idea",
      run: () => openQuickAdd("idea"),
    },
    {
      id: "action:prompt",
      label: "Add prompt",
      group: "Create",
      keywords: "new workflow",
      run: () => openQuickAdd("prompt"),
    },
    {
      id: "action:media",
      label: "Add media title",
      group: "Create",
      keywords: "new library anime manga game movie show album",
      run: () => openQuickAdd("media"),
    },
    {
      id: "action:shortcuts",
      label: "Keyboard shortcuts",
      group: "Help",
      keywords: "keys help",
      run: () => setShortcuts(true),
    },
  ];

  const view = timer.data;
  const focus: Command[] = view
    ? [
        view.status === "paused"
          ? { id: "focus:resume", label: "Resume focus", run: () => resume.mutate(undefined) }
          : { id: "focus:pause", label: "Pause focus", run: () => pause.mutate(undefined) },
        { id: "focus:stop", label: "Stop focus", run: () => stop.mutate(undefined) },
      ].map((command) => ({ ...command, group: "Focus", keywords: "timer pomodoro" }))
    : [
        ...(due.data ?? []).map((task) => ({
          id: `focus:${task.id}`,
          label: `Start focus: ${task.title}`,
          group: "Focus",
          keywords: `timer pomodoro ${task.project_name}`,
          run: () => start.mutate(task.id),
        })),
        // a timer always runs on a task, so other tasks are started from their project
        {
          id: "focus:pick",
          label: "Start focus on another task",
          group: "Focus",
          keywords: "timer pomodoro",
          run: () => navigate("/projects"),
        },
      ];

  const projectCommands: Command[] = (projects.data ?? []).map((project) => ({
    id: `project:${project.id}`,
    label: project.name,
    group: "Projects",
    keywords: project.tags.join(" "),
    run: () => navigate(`/projects/${project.id}`),
  }));

  const mediaCommands: Command[] = (media.data ?? []).map((entry) => ({
    id: `media:${entry.id}`,
    label: entry.title,
    group: "Media",
    keywords: TYPE_LABELS[entry.media_type],
    searchOnly: true,
    run: () => navigate(`/media?open=${entry.id}`),
  }));

  return [...actions, ...focus, ...pages, ...projectCommands, ...mediaCommands];
}
