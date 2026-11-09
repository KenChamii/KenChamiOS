import { Outlet, useLocation } from "react-router-dom";
import { TimerBar } from "@/features/focus/timer-bar";
import { useRefreshOnSync } from "@/hooks/use-sync";
import { Sidebar } from "./sidebar";

export function Layout() {
  useRefreshOnSync();
  const { pathname } = useLocation();
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TimerBar />
        <main className="flex-1 overflow-auto px-8 pb-10 pt-6">
          <div key={pathname} className="mx-auto max-w-6xl animate-page-in">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
