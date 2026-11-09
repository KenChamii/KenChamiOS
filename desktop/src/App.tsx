import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Layout } from "@/components/layout/layout";
import { DashboardPage } from "@/pages/dashboard";
import { MediaPage } from "@/pages/media";
import { ProjectPage } from "@/pages/project";
import { ProjectsPage } from "@/pages/projects";
import { SettingsPage } from "@/pages/settings";
import { WorkflowPage } from "@/pages/workflow";

const queryClient = new QueryClient();

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { path: "/", element: <DashboardPage /> },
      { path: "/projects", element: <ProjectsPage /> },
      { path: "/projects/:id", element: <ProjectPage /> },
      { path: "/workflow", element: <WorkflowPage /> },
      { path: "/media", element: <MediaPage /> },
      { path: "/settings", element: <SettingsPage /> },
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
