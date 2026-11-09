import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { STAGE_LABELS } from "@/features/projects/meta";
import { useDeleteSavePoint } from "@/hooks/use-save-points";
import type { SavePoint } from "@/lib/api/save-points";
import { errorMessage } from "@/lib/errors";
import { commitLabel } from "./meta";
import { SavePointBody } from "./save-point-body";

type SavePointEntryProps = { savePoint: SavePoint; initiallyOpen: boolean };

export function SavePointEntry({ savePoint, initiallyOpen }: SavePointEntryProps) {
  const [open, setOpen] = useState(initiallyOpen);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const remove = useDeleteSavePoint(savePoint.project_id);
  const git = commitLabel(savePoint.git_branch, savePoint.git_commit);
  const Chevron = open ? ChevronDown : ChevronRight;

  return (
    <li className="rounded-lg border border-border bg-card">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="flex w-full items-center gap-3 p-3 text-left"
      >
        <Chevron size={16} className="shrink-0 text-muted-foreground" />
        <span className="flex-1 font-medium">{savePoint.note}</span>
        <Badge>{STAGE_LABELS[savePoint.stage]}</Badge>
        <span className="text-xs text-muted-foreground">
          {new Date(savePoint.created_at).toLocaleString()}
        </span>
      </button>
      {open && (
        <div className="flex flex-col gap-3 border-t border-border p-3">
          {git && <p className="font-mono text-xs text-muted-foreground">{git}</p>}
          <SavePointBody title="Done" source={savePoint.done_md} />
          <SavePointBody title="Next" source={savePoint.next_md} />
          {remove.error && <p className="text-sm text-destructive">{errorMessage(remove.error)}</p>}
          <div className="flex gap-2">
            {confirmingDelete ? (
              <>
                <Button
                  variant="destructive"
                  size="sm"
                  disabled={remove.isPending}
                  onClick={() => remove.mutate(savePoint.id)}
                >
                  Confirm delete
                </Button>
                <Button variant="ghost" size="sm" onClick={() => setConfirmingDelete(false)}>
                  Cancel
                </Button>
              </>
            ) : (
              <Button variant="ghost" size="sm" onClick={() => setConfirmingDelete(true)}>
                Delete
              </Button>
            )}
          </div>
        </div>
      )}
    </li>
  );
}
