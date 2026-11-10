import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Layout } from "@/components/layout/layout";
import { DashboardPage } from "@/pages/dashboard";
import { HabitsPage } from "@/pages/habits";
import { InboxPage } from "@/pages/inbox";
import { MediaPage } from "@/pages/media";
import { NotesPage } from "@/pages/notes";
import { PlannerPage } from "@/pages/planner";
import { ProjectPage } from "@/pages/project";
import { ProjectsPage } from "@/pages/projects";
import { RecapPage } from "@/pages/recap";
import { SettingsPage } from "@/pages/settings";
import { TrashPage } from "@/pages/trash";
import { WeeklyReviewPage } from "@/pages/weekly-review";
import { WorkflowPage } from "@/pages/workflow";

const queryClient = new QueryClient();

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { path: "/", element: <DashboardPage /> },
      { path: "/inbox", element: <InboxPage /> },
      { path: "/planner", element: <PlannerPage /> },
      { path: "/notes", element: <NotesPage /> },
      { path: "/review", element: <WeeklyReviewPage /> },
      { path: "/habits", element: <HabitsPage /> },
      { path: "/trash", element: <TrashPage /> },
      { path: "/projects", element: <ProjectsPage /> },
      { path: "/projects/:id/:tab?", element: <ProjectPage /> },
      { path: "/workflow", element: <WorkflowPage /> },
      { path: "/media", element: <MediaPage /> },
      { path: "/recap", element: <RecapPage /> },
      { path: "/settings/:section?", element: <SettingsPage /> },
    ],
  },
]);

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
