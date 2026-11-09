import { Outlet } from "react-router-dom";
import { TimerBar } from "@/features/focus/timer-bar";
import { Sidebar } from "./sidebar";

export function Layout() {
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TimerBar />
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
