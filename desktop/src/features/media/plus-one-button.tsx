import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSetMediaProgress } from "@/hooks/use-media";
import type { LibraryEntry } from "@/lib/api/media";
import { PROGRESS_LABELS } from "./meta";

export function PlusOneButton({ entry }: { entry: LibraryEntry }) {
  const setProgress = useSetMediaProgress();
  const finished = entry.progress_total !== null && entry.progress >= entry.progress_total;
  const label = `+1 ${PROGRESS_LABELS[entry.media_type].toLowerCase()}`;

  return (
    <Button
      size="sm"
      variant="outline"
      aria-label={label}
      title={label}
      disabled={finished || setProgress.isPending}
      onClick={() => setProgress.mutate({ id: entry.id, value: entry.progress + 1 })}
    >
      <Plus size={14} />1
    </Button>
  );
}
