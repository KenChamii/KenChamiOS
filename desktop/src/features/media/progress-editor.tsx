import { Minus, Plus } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useSetMediaProgress } from "@/hooks/use-media";
import type { LibraryEntry } from "@/lib/api/media";
import { errorMessage } from "@/lib/errors";
import { PROGRESS_LABELS } from "./meta";

/** Progress is saved as soon as it changes, because each change is logged. */
export function ProgressEditor({ entry }: { entry: LibraryEntry }) {
  const setProgress = useSetMediaProgress();
  const [text, setText] = useState(String(entry.progress));
  const max = entry.progress_total;

  function save(value: number) {
    if (Number.isInteger(value) && value >= 0 && (max === null || value <= max)) {
      setProgress.mutate({ id: entry.id, value });
    }
  }

  return (
    <div className="flex flex-col gap-1">
      <span className="text-sm font-medium">{PROGRESS_LABELS[entry.media_type]}</span>
      <div className="flex items-center gap-2">
        <Button
          size="icon"
          variant="outline"
          aria-label="Minus one"
          disabled={entry.progress === 0 || setProgress.isPending}
          onClick={() => save(entry.progress - 1)}
        >
          <Minus size={16} />
        </Button>
        <Input
          type="number"
          min={0}
          max={max ?? undefined}
          className="w-24"
          aria-label={PROGRESS_LABELS[entry.media_type]}
          value={text}
          onChange={(event) => setText(event.target.value)}
          onBlur={() => save(Number(text))}
        />
        <Button
          size="icon"
          variant="outline"
          aria-label="Plus one"
          disabled={(max !== null && entry.progress >= max) || setProgress.isPending}
          onClick={() => save(entry.progress + 1)}
        >
          <Plus size={16} />
        </Button>
        {max !== null && <span className="text-sm text-muted-foreground">of {max}</span>}
      </div>
      {setProgress.error && (
        <p className="text-sm text-destructive">{errorMessage(setProgress.error)}</p>
      )}
    </div>
  );
}
