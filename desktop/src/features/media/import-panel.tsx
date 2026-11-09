import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { useConfirmImport, usePreviewImport } from "@/hooks/use-media-transfer";
import type { ImportFormat } from "@/lib/api/media-transfer";
import { errorMessage } from "@/lib/errors";
import { GENERIC_STATUS_LABELS, TYPE_LABELS } from "./meta";

const FORMATS: Record<ImportFormat, { label: string; extensions: string[] }> = {
  csv: { label: "CSV file", extensions: ["csv"] },
  mal: { label: "MyAnimeList / AniList export (XML)", extensions: ["xml"] },
};

export function ImportPanel() {
  const [format, setFormat] = useState<ImportFormat>("csv");
  const [path, setPath] = useState<string | null>(null);
  const preview = usePreviewImport();
  const confirm = useConfirmImport();

  async function chooseFile() {
    const chosen = await open({
      multiple: false,
      directory: false,
      filters: [{ name: FORMATS[format].label, extensions: FORMATS[format].extensions }],
    });
    if (!chosen) return;
    setPath(chosen);
    confirm.reset();
    preview.mutate({ path: chosen, format });
  }

  const data = preview.data;
  const error = preview.error ?? confirm.error;

  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-medium">Import</h3>
      <p className="text-sm text-muted-foreground">
        Pick a file to see what would be added first. Nothing is written until you confirm, and
        titles already in the library are skipped. AniList lists need to be exported to MyAnimeList
        XML with a converter tool.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Select
          aria-label="File format"
          className="w-56"
          value={format}
          options={(Object.keys(FORMATS) as ImportFormat[]).map((id) => ({
            value: id,
            label: FORMATS[id].label,
          }))}
          onChange={(next) => {
            setFormat(next);
            setPath(null);
            preview.reset();
            confirm.reset();
          }}
        />
        <Button variant="outline" disabled={preview.isPending} onClick={chooseFile}>
          Choose file
        </Button>
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {data && path && !confirm.data && (
        <>
          <p className="text-sm">
            {data.new_count} new, {data.duplicate_count} already in the library,{" "}
            {data.rejected.length} rejected.
          </p>
          <ul className="max-h-48 divide-y divide-border overflow-auto rounded-xl bg-muted">
            {data.rows.map((row) => (
              <li key={row.line} className="flex items-center gap-2 px-3 py-1.5 text-sm">
                <span className="min-w-0 flex-1 truncate">{row.title}</span>
                <span className="text-xs text-muted-foreground">
                  {TYPE_LABELS[row.media_type]} · {GENERIC_STATUS_LABELS[row.status]}
                </span>
                {row.duplicate && <Badge>Skipped</Badge>}
              </li>
            ))}
          </ul>
          {data.rejected.length > 0 && (
            <ul className="max-h-32 overflow-auto text-xs text-destructive">
              {data.rejected.map((row) => (
                <li key={row.line}>
                  Row {row.line}: {row.reason}
                </li>
              ))}
            </ul>
          )}
          <Button
            disabled={data.new_count === 0 || confirm.isPending}
            onClick={() => confirm.mutate({ path, format })}
          >
            Import {data.new_count} titles
          </Button>
        </>
      )}
      {confirm.data && (
        <p className="text-sm">
          Added {confirm.data.added}, skipped {confirm.data.duplicates} duplicates, rejected{" "}
          {confirm.data.rejected}.
        </p>
      )}
    </section>
  );
}
