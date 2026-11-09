import { Search } from "lucide-react";
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/cn";
import { useOverlays } from "@/stores/overlays";
import { filterCommands, type Command } from "./filter";
import { useCommands } from "./use-commands";

export function CommandPalette() {
  const { palette, closePalette } = useOverlays();
  return (
    <Dialog open={palette.open} onOpenChange={(open) => !open && closePalette()}>
      <DialogContent
        aria-describedby={undefined}
        className="top-[15%] max-w-xl translate-y-0 overflow-hidden p-0"
      >
        <DialogTitle className="sr-only">Command palette</DialogTitle>
        {palette.open && <PaletteBody initialQuery={palette.query} onDone={closePalette} />}
      </DialogContent>
    </Dialog>
  );
}

function PaletteBody({ initialQuery, onDone }: { initialQuery: string; onDone: () => void }) {
  const [query, setQuery] = useState(initialQuery);
  const [active, setActive] = useState(0);
  const listRef = useRef<HTMLUListElement>(null);
  const id = useId();
  const results = filterCommands(useCommands(), query);
  const current = Math.min(active, results.length - 1);

  useEffect(() => {
    listRef.current?.children[current]?.scrollIntoView({ block: "nearest" });
  }, [current]);

  function run(command: Command | undefined) {
    if (!command) return;
    onDone();
    command.run();
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActive(Math.min(current + 1, results.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive(Math.max(current - 1, 0));
    } else if (event.key === "Enter") {
      event.preventDefault();
      run(results[current]);
    }
  }

  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-3 border-b border-border px-4">
        <Search size={18} className="shrink-0 text-muted-foreground" />
        <input
          autoFocus
          role="combobox"
          aria-expanded
          aria-controls={`${id}-list`}
          aria-activedescendant={results.length > 0 ? `${id}-${current}` : undefined}
          aria-label="Search commands, projects and media"
          placeholder="Type a command, project or title"
          className="h-14 flex-1 bg-transparent text-base outline-none placeholder:text-muted-foreground"
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setActive(0);
          }}
          onKeyDown={handleKey}
        />
      </div>
      {results.length === 0 ? (
        <p className="px-4 py-8 text-center text-sm text-muted-foreground">Nothing matches.</p>
      ) : (
        <ul ref={listRef} id={`${id}-list`} role="listbox" className="max-h-96 overflow-auto p-2">
          {results.map((command, index) => (
            <li
              key={command.id}
              id={`${id}-${index}`}
              role="option"
              aria-selected={index === current}
              onMouseMove={() => setActive(index)}
              onClick={() => run(command)}
              className={cn(
                "flex cursor-pointer items-center justify-between gap-4 rounded-xl px-3 py-2 text-sm",
                index === current && "bg-muted",
              )}
            >
              <span className="truncate">{command.label}</span>
              <span className="shrink-0 text-label text-muted-foreground">{command.group}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
