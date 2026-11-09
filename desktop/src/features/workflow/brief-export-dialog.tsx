import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { useWriteBrief } from "@/hooks/use-brief";
import type { BriefPreview } from "@/lib/api/brief";
import { errorMessage } from "@/lib/errors";
import { DiffView } from "./diff-view";

type BriefExportDialogProps = {
  projectId: string;
  preview: BriefPreview | null;
  onClose: () => void;
};

export function BriefExportDialog({ projectId, preview, onClose }: BriefExportDialogProps) {
  const write = useWriteBrief(projectId);

  return (
    <Dialog open={preview !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent aria-describedby={undefined} className="max-w-3xl">
        <DialogTitle className="mb-4 text-lg font-semibold">Export brief</DialogTitle>
        {preview && (
          <div className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">
              {preview.file_exists
                ? `The brief goes into a section marked by kenchamios comments in ${preview.target}; the rest of the file stays as it is.`
                : `${preview.target} does not exist yet and will be created.`}
            </p>
            {preview.changed ? (
              <DiffView lines={preview.diff} />
            ) : (
              <p className="text-sm">The file is already up to date.</p>
            )}
            {write.error && <p className="text-sm text-destructive">{errorMessage(write.error)}</p>}
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={onClose}>
                Cancel
              </Button>
              <Button
                disabled={!preview.changed || write.isPending}
                onClick={() => write.mutate(undefined, { onSuccess: onClose })}
              >
                Write to {preview.target}
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
