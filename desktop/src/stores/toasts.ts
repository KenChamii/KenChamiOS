import { create } from "zustand";

export type Toast = {
  id: number;
  message: string;
  /** reverses the action the toast reports */
  undo?: () => Promise<unknown>;
};

type ToastState = {
  toast: Toast | null;
  show: (message: string, undo?: () => Promise<unknown>) => void;
  dismiss: (id: number) => void;
};

let nextId = 1;

/** One toast at a time; a new one replaces the last, like a status line. */
export const useToasts = create<ToastState>()((set) => ({
  toast: null,
  show: (message, undo) => set({ toast: { id: nextId++, message, undo } }),
  dismiss: (id) => set((state) => (state.toast?.id === id ? { toast: null } : state)),
}));
