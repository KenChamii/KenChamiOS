import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useRevokeDevice, useSetSyncEnabled, useSyncStatus } from "@/hooks/use-sync";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import type { PairedDevice } from "@/lib/api/sync";
import { PairingDialog } from "./pairing-dialog";
import { groupFingerprint } from "./pairing-text";

export function SyncCard() {
  const status = useSyncStatus();
  const setEnabled = useSetSyncEnabled();
  const revoke = useRevokeDevice();
  const [pairing, setPairing] = useState(false);

  if (status.error) {
    return <p className="text-sm text-destructive">{errorMessage(status.error)}</p>;
  }
  const data = status.data;
  if (!data) return null;
  const error = setEnabled.error ?? revoke.error;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">Phone sync</h2>
      <p className="text-sm text-muted-foreground">
        Keeps this PC and your phone in sync over your home Wi-Fi. Nothing leaves your network. The
        phone starts every sync: every 15 minutes and after each change.
      </p>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={data.enabled}
          disabled={setEnabled.isPending}
          onChange={(event) => setEnabled.mutate(event.target.checked)}
        />
        Allow paired phones to sync with this PC
      </label>
      {data.running && data.address && (
        <p className="text-sm">
          Listening on <span className="font-mono">{data.address}</span>
          {data.fingerprint && (
            <span className="block break-all font-mono text-xs text-muted-foreground">
              Fingerprint {groupFingerprint(data.fingerprint)}
            </span>
          )}
        </p>
      )}
      {data.enabled && !data.running && (
        <p className="text-sm text-destructive">Not running. Check that you are on Wi-Fi.</p>
      )}
      {data.last_error && (
        <p className="text-sm text-destructive">
          Last problem {timeAgo(data.last_error.at)}: {data.last_error.message}
        </p>
      )}
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <Devices
        devices={data.devices}
        revoking={revoke.isPending}
        onRevoke={(id) => revoke.mutate(id)}
      />
      <div>
        <Button disabled={!data.running} onClick={() => setPairing(true)}>
          Pair a phone
        </Button>
      </div>
      <PairingDialog open={pairing} onClose={() => setPairing(false)} />
    </Card>
  );
}

function Devices({
  devices,
  revoking,
  onRevoke,
}: {
  devices: PairedDevice[];
  revoking: boolean;
  onRevoke: (deviceId: string) => void;
}) {
  if (devices.length === 0) {
    return <p className="text-sm text-muted-foreground">No phone is paired yet.</p>;
  }
  return (
    <ul className="flex flex-col divide-y divide-border text-sm">
      {devices.map((device) => (
        <li key={device.device_id} className="flex items-center justify-between gap-3 py-2">
          <div>
            <p className="font-medium">{device.name}</p>
            <p className="text-muted-foreground">
              {device.last_sync_at ? `Last sync ${timeAgo(device.last_sync_at)}` : "Never synced"}
            </p>
          </div>
          <Button
            variant="destructive"
            size="sm"
            disabled={revoking}
            onClick={() => onRevoke(device.device_id)}
          >
            Revoke
          </Button>
        </li>
      ))}
    </ul>
  );
}
