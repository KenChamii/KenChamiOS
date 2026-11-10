import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { NAV_PAGES } from "@/components/layout/nav-pages";
import { IDLE, readShortcut } from "@/lib/shortcuts";
import { useOverlays } from "@/stores/overlays";

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
  );
}

/** The app-wide keys: Ctrl+K, Ctrl+N, Ctrl+Shift+P, ? and G then a page letter. */
export function useShortcuts() {
  const navigate = useNavigate();

  useEffect(() => {
    let state = IDLE;
    function onKeyDown(event: KeyboardEvent) {
      const result = readShortcut(
        {
          key: event.key,
          ctrlKey: event.ctrlKey,
          metaKey: event.metaKey,
          altKey: event.altKey,
          shiftKey: event.shiftKey,
          typing: isTyping(event.target),
          at: event.timeStamp,
        },
        state,
      );
      state = result.state;
      const action = result.action;
      if (!action) return;

      const overlays = useOverlays.getState();
      if (action.type === "go") {
        const page = NAV_PAGES.find((candidate) => candidate.key === action.key);
        if (!page) return;
        navigate(page.to);
      } else if (action.type === "palette") {
        overlays.openPalette();
      } else if (action.type === "quick-add") {
        overlays.openQuickAdd("task");
      } else if (action.type === "plan-day") {
        overlays.openDayFlow("plan");
      } else {
        overlays.setShortcuts(true);
      }
      event.preventDefault();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [navigate]);
}
