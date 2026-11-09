import { create } from "zustand";

export type QuickAddMode = "task" | "idea" | "prompt" | "media";

type OverlayState = {
  palette: { open: boolean; query: string };
  quickAdd: QuickAddMode | null;
  shortcuts: boolean;
  openPalette: (query?: string) => void;
  closePalette: () => void;
  openQuickAdd: (mode: QuickAddMode) => void;
  closeQuickAdd: () => void;
  setShortcuts: (open: boolean) => void;
};

/** The app-wide dialogs, so shortcuts, the sidebar and the dashboard can open them. */
export const useOverlays = create<OverlayState>()((set) => ({
  palette: { open: false, query: "" },
  quickAdd: null,
  shortcuts: false,
  openPalette: (query = "") => set({ palette: { open: true, query } }),
  closePalette: () => set({ palette: { open: false, query: "" } }),
  openQuickAdd: (mode) => set({ quickAdd: mode }),
  closeQuickAdd: () => set({ quickAdd: null }),
  setShortcuts: (shortcuts) => set({ shortcuts }),
}));
