import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  useBackupStatus,
  useConfigureBackups,
  useRestoreBackup,
  useRunBackup,
} from "@/hooks/use-backups";
import type { BackupFile, BackupStatus } from "@/lib/api/backups";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { formatSize, parseKeep } from "./format";

export function BackupCard() {
  const status = useBackupStatus();
  if (status.error) return <p className="text-sm text-destructive">{errorMessage(status.error)}</p>;
  if (!status.data) return null;
  return (
    <>
      <BackupSettings status={status.data} />
      <BackupList copies={status.data.copies} />
    </>
  );
}

function BackupSettings({ status }: { status: BackupStatus }) {
  const configure = useConfigureBackups();
  const run = useRunBackup();
  const [keep, setKeep] = useState(String(status.keep));
  const parsedKeep = parseKeep(keep);
  const error = configure.error ?? run.error;

  async function chooseFolder() {
    const folder = await open({ directory: true, multiple: false });
    if (folder) configure.mutate({ folder, keep: status.keep });
  }

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="font-semibold tracking-tight">Daily backups</h2>
        <p className="text-sm text-muted-foreground">
          A full copy of the database once a day while the app runs. Pick a folder on another drive
          or a synced folder to keep it safe.
        </p>
      </div>
      <div className="flex flex-col gap-1.5">
        <span className="text-label text-muted-foreground">Folder</span>
        <p className="break-all text-sm">{status.folder}</p>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void chooseFolder()}>
            Choose folder
          </Button>
          {status.custom_folder && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => configure.mutate({ folder: null, keep: status.keep })}
            >
              Use the default folder
            </Button>
          )}
        </div>
      </div>
      <form
        className="flex flex-wrap items-end gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (parsedKeep !== null) {
            const folder = status.custom_folder ? status.folder : null;
            configure.mutate({ folder, keep: parsedKeep });
          }
        }}
      >
        <Field label="Backups to keep">
          <Input
            type="number"
            min={1}
            max={365}
            required
            className="w-40"
            value={keep}
            onChange={(event) => setKeep(event.target.value)}
          />
        </Field>
        <Button
          type="submit"
          variant="outline"
          disabled={configure.isPending || parsedKeep === null}
        >
          Save
        </Button>
      </form>
      <div className="flex flex-wrap items-center gap-3 text-sm">
        <Button disabled={run.isPending} onClick={() => run.mutate()}>
          {run.isPending ? "Backing up…" : "Back up now"}
        </Button>
        <span className="text-muted-foreground">
          {status.last_at ? `Last backup ${timeAgo(status.last_at)}` : "No backup yet"}
        </span>
      </div>
      {status.last_error && (
        <p className="text-sm text-destructive">Last backup failed: {status.last_error}</p>
      )}
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </Card>
  );
}

function BackupList({ copies }: { copies: BackupFile[] }) {
  const restore = useRestoreBackup();
  const [confirming, setConfirming] = useState<string | null>(null);

  return (
    <Card className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h2 className="font-semibold tracking-tight">Restore</h2>
        <p className="text-sm text-muted-foreground">
          Restoring replaces everything with the copy and restarts the app. The current database is
          kept next to it as kenchamios.db.before-restore.
        </p>
      </div>
      {copies.length === 0 ? (
        <p className="text-sm text-muted-foreground">No backups in this folder yet.</p>
      ) : (
        <ul className="flex flex-col divide-y divide-border">
          {copies.map((copy) => (
            <li key={copy.path} className="flex items-center gap-3 py-2 text-sm">
              <span className="flex-1">
                {copy.created_at ? new Date(copy.created_at).toLocaleString() : copy.name}
              </span>
              <span className="text-muted-foreground">{formatSize(copy.size_bytes)}</span>
              {confirming === copy.path ? (
                <>
                  <Button
                    size="sm"
                    variant="destructive"
                    disabled={restore.isPending}
                    onClick={() => restore.mutate(copy.path)}
                  >
                    Replace and restart
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setConfirming(null)}>
                    Cancel
                  </Button>
                </>
              ) : (
                <Button size="sm" variant="outline" onClick={() => setConfirming(copy.path)}>
                  Restore
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
      {restore.error && <p className="text-sm text-destructive">{errorMessage(restore.error)}</p>}
    </Card>
  );
}
