import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Theme = "light" | "dark" | "system";

type ThemeState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

export const useThemeStore = create<ThemeState>()(
  persist((set) => ({ theme: "system", setTheme: (theme) => set({ theme }) }), { name: "theme" }),
);

const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function applyTheme(theme: Theme) {
  const dark = theme === "dark" || (theme === "system" && darkQuery.matches);
  document.documentElement.classList.toggle("dark", dark);
}

export function watchTheme() {
  const apply = () => applyTheme(useThemeStore.getState().theme);
  apply();
  useThemeStore.subscribe(apply);
  darkQuery.addEventListener("change", apply);
}
