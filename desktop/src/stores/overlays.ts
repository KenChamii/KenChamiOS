import { create } from "zustand";

export type QuickAddMode = "task" | "idea" | "prompt" | "media";

/** The morning plan or the evening review of today. */
export type DayFlow = "plan" | "review";

type OverlayState = {
  palette: { open: boolean; query: string };
  quickAdd: QuickAddMode | null;
  /** text to start the quick add with, e.g. what was typed in the palette */
  quickAddText: string;
  shortcuts: boolean;
  dayFlow: DayFlow | null;
  openPalette: (query?: string) => void;
  closePalette: () => void;
  openQuickAdd: (mode: QuickAddMode, text?: string) => void;
  closeQuickAdd: () => void;
  setShortcuts: (open: boolean) => void;
  openDayFlow: (flow: DayFlow) => void;
  closeDayFlow: () => void;
};

/** The app-wide dialogs, so shortcuts, the sidebar and the dashboard can open them. */
export const useOverlays = create<OverlayState>()((set) => ({
  palette: { open: false, query: "" },
  quickAdd: null,
  quickAddText: "",
  shortcuts: false,
  dayFlow: null,
  openPalette: (query = "") => set({ palette: { open: true, query } }),
  closePalette: () => set({ palette: { open: false, query: "" } }),
  openQuickAdd: (mode, text = "") => set({ quickAdd: mode, quickAddText: text }),
  closeQuickAdd: () => set({ quickAdd: null }),
  setShortcuts: (shortcuts) => set({ shortcuts }),
  openDayFlow: (dayFlow) => set({ dayFlow }),
  closeDayFlow: () => set({ dayFlow: null }),
}));
