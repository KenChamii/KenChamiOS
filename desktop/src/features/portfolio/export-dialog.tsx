import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { MarkdownPreview } from "@/features/docs/markdown-preview";
import { useExportPortfolio } from "@/hooks/use-portfolio";
import { errorMessage } from "@/lib/errors";

type ExportDialogProps = {
  projectId: string;
  markdown: string | null;
  onClose: () => void;
};

export function ExportDialog({ projectId, markdown, onClose }: ExportDialogProps) {
  const exportPage = useExportPortfolio(projectId);

  async function chooseFolderAndExport() {
    const folder = await open({ directory: true, multiple: false });
    if (folder) exportPage.mutate(folder);
  }

  return (
    <Dialog open={markdown !== null} onOpenChange={(next) => !next && onClose()}>
      <DialogContent aria-describedby={undefined} className="max-w-3xl">
        <DialogTitle className="mb-4 text-lg font-semibold">Export portfolio page</DialogTitle>
        {markdown !== null && (
          <div className="flex flex-col gap-4">
            <div className="max-h-[60vh] overflow-auto">
              <MarkdownPreview source={markdown} />
            </div>
            <p className="text-sm text-muted-foreground">
              Pick a folder. The page is written to portfolio.md inside a folder named after the
              project, and the screenshots are copied to its images folder.
            </p>
            {exportPage.error && (
              <p className="text-sm text-destructive">{errorMessage(exportPage.error)}</p>
            )}
            {exportPage.data && (
              <p className="text-sm">
                Exported to {exportPage.data.output_dir} with {exportPage.data.screenshots}{" "}
                screenshots.
              </p>
            )}
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={onClose}>
                Close
              </Button>
              <Button disabled={exportPage.isPending} onClick={chooseFolderAndExport}>
                Choose folder and export
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
