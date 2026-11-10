import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { useOverlays } from "@/stores/overlays";
import { NAV_PAGES } from "./nav-pages";

const SHORTCUTS: { keys: string[]; label: string }[] = [
  { keys: ["Ctrl", "K"], label: "Command palette" },
  { keys: ["Ctrl", "N"], label: "Quick add" },
  { keys: ["Ctrl", "Shift", "P"], label: "Plan my day" },
  ...NAV_PAGES.map((page) => ({ keys: ["G", page.key.toUpperCase()], label: page.label })),
  { keys: ["?"], label: "This list" },
];

export function ShortcutsDialog() {
  const { shortcuts, setShortcuts } = useOverlays();
  return (
    <Dialog open={shortcuts} onOpenChange={setShortcuts}>
      <DialogContent aria-describedby={undefined}>
        <DialogTitle className="mb-4 text-lg font-semibold">Keyboard shortcuts</DialogTitle>
        <dl className="flex flex-col gap-2 text-sm">
          {SHORTCUTS.map((shortcut) => (
            <div key={shortcut.label} className="flex items-center justify-between gap-4">
              <dt>{shortcut.label}</dt>
              <dd className="flex gap-1">
                {shortcut.keys.map((key) => (
                  <kbd
                    key={key}
                    className="min-w-7 rounded-md bg-muted px-1.5 py-0.5 text-center font-sans text-xs font-medium"
                  >
                    {key}
                  </kbd>
                ))}
              </dd>
            </div>
          ))}
        </dl>
        <p className="mt-4 text-xs text-muted-foreground">
          Press G, then the letter. Letters are ignored while you type in a field.
        </p>
      </DialogContent>
    </Dialog>
  );
}
