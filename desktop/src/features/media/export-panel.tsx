import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { useExportCsv, useExportExists } from "@/hooks/use-media-transfer";
import { errorMessage } from "@/lib/errors";

export function ExportPanel() {
  const exists = useExportExists();
  const write = useExportCsv();
  // set while the chosen folder already holds an export that would be replaced
  const [pending, setPending] = useState<string | null>(null);

  async function chooseFolder() {
    const folder = await open({ directory: true, multiple: false });
    if (!folder) return;
    write.reset();
    setPending(null);
    exists.mutate(folder, {
      onSuccess: (taken) => {
        if (taken) setPending(folder);
        else write.mutate({ folder, overwrite: false });
      },
    });
  }

  const error = exists.error ?? write.error;

  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-medium">Export</h3>
      <p className="text-sm text-muted-foreground">
        Writes the whole library to kenchamios-media.csv in a folder you choose. The file can be
        imported again here.
      </p>
      <div>
        <Button
          variant="outline"
          disabled={exists.isPending || write.isPending}
          onClick={chooseFolder}
        >
          Choose folder and export
        </Button>
      </div>
      {pending && (
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <span>kenchamios-media.csv already exists there. Replace it?</span>
          <Button
            size="sm"
            variant="destructive"
            onClick={() => {
              write.mutate({ folder: pending, overwrite: true });
              setPending(null);
            }}
          >
            Replace
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setPending(null)}>
            Cancel
          </Button>
        </div>
      )}
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {write.data && <p className="break-all text-sm">Exported to {write.data}</p>}
    </section>
  );
}
