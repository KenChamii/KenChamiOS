import { Outlet, useLocation } from "react-router-dom";
import { CommandPalette } from "@/features/palette/command-palette";
import { QuickAddDialog } from "@/features/quick-add/quick-add-dialog";
import { useShortcuts } from "@/hooks/use-shortcuts";
import { useRefreshOnSync } from "@/hooks/use-sync";
import { ShortcutsDialog } from "./shortcuts-dialog";
import { Sidebar } from "./sidebar";
import { Toaster } from "./toaster";

export function Layout() {
  useRefreshOnSync();
  useShortcuts();
  // animate between pages, not between the tabs of one project
  const page = useLocation().pathname.split("/").slice(0, 3).join("/");
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <main className="flex-1 overflow-auto px-8 pb-10 pt-6">
          <div key={page} className="mx-auto max-w-6xl animate-page-in">
            <Outlet />
          </div>
        </main>
      </div>
      <CommandPalette />
      <QuickAddDialog />
      <ShortcutsDialog />
      <Toaster />
    </div>
  );
}
