import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ExportPanel } from "./export-panel";
import { ImportPanel } from "./import-panel";

type TransferDialogProps = { open: boolean; onClose: () => void };

export function TransferDialog({ open, onClose }: TransferDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent aria-describedby={undefined} className="max-h-[85vh] max-w-2xl overflow-auto">
        <DialogTitle className="mb-4 text-lg font-semibold">Import and export</DialogTitle>
        {open && (
          <div className="flex flex-col gap-6">
            <ImportPanel />
            <ExportPanel />
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
