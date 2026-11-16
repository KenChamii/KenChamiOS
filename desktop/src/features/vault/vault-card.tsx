import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import {
  useConfigureVault,
  usePreviewVault,
  useSyncVault,
  useVaultStatus,
} from "@/hooks/use-vault";
import type { VaultStatus } from "@/lib/api/vault";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { VaultReportView } from "./vault-report";

export function VaultCard() {
  const status = useVaultStatus();
  if (status.error) return <p className="text-sm text-destructive">{errorMessage(status.error)}</p>;
  if (!status.data) return null;
  return <VaultForm status={status.data} />;
}

function VaultForm({ status }: { status: VaultStatus }) {
  const configure = useConfigureVault();
  const preview = usePreviewVault();
  const sync = useSyncVault();
  const [path, setPath] = useState(status.path);
  const [excluded, setExcluded] = useState(status.excluded.join("\n"));
  const names = excluded.split("\n").filter((line) => line.trim() !== "");
  const save = (enabled: boolean) => configure.mutate({ path, enabled, excluded: names });
  const error = configure.error ?? preview.error ?? sync.error;
  const report = sync.data ?? preview.data;

  async function pick() {
    const folder = await open({ directory: true, multiple: false });
    if (typeof folder === "string") setPath(folder);
  }

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="font-semibold tracking-tight">Obsidian vault</h2>
        <p className="text-sm text-muted-foreground">
          Daily notes, project ideas and plans, and task ticks stay in step with the{" "}
          <code>KenChamiOS</code> folder of your vault. Back the vault up before the first sync.
        </p>
      </div>
      <Field label="Vault folder">
        <div className="flex gap-2">
          <Input value={path} onChange={(event) => setPath(event.target.value)} />
          <Button variant="outline" onClick={() => void pick()}>
            Choose
          </Button>
        </div>
      </Field>
      {!status.exists && (
        <p className="text-sm text-muted-foreground">That folder does not exist yet.</p>
      )}
      <Field label="Keep these projects out of the vault (one name per line)">
        <Textarea value={excluded} onChange={(event) => setExcluded(event.target.value)} />
      </Field>
      <label className="flex items-center justify-between gap-3 text-sm">
        <span>Sync about every 30 seconds while the app is open</span>
        <Switch
          aria-label="Sync automatically"
          checked={status.enabled}
          disabled={configure.isPending}
          onCheckedChange={save}
        />
      </label>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="outline"
          disabled={configure.isPending}
          onClick={() => save(status.enabled)}
        >
          Save
        </Button>
        <Button variant="outline" disabled={preview.isPending} onClick={() => preview.mutate()}>
          Preview
        </Button>
        <Button disabled={sync.isPending} onClick={() => sync.mutate()}>
          Write to vault now
        </Button>
        <span className="text-sm text-muted-foreground">
          {status.last_sync ? `Last sync ${timeAgo(status.last_sync)}` : "Never synced"}
        </span>
      </div>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {report && <VaultReportView report={report} applied={sync.data !== undefined} />}
    </Card>
  );
}
