import { invoke } from "@tauri-apps/api/core";

export type PairedDevice = {
  device_id: string;
  name: string;
  paired_at: number;
  last_sync_at: number | null;
  pulled_seq: number;
};

export type SyncStatus = {
  enabled: boolean;
  running: boolean;
  address: string | null;
  fingerprint: string | null;
  pairing_open: boolean;
  last_error: { at: number; message: string } | null;
  devices: PairedDevice[];
};

export type PairingPayload = {
  v: number;
  host: string;
  port: number;
  fp: string;
  code: string;
};

export type PairingInfo = {
  qr_svg: string;
  payload: PairingPayload;
  expires_in_s: number;
};

export function getSyncStatus() {
  return invoke<SyncStatus>("sync_status");
}

export function setSyncEnabled(enabled: boolean) {
  return invoke<void>("sync_set_enabled", { enabled });
}

export function startPairing() {
  return invoke<PairingInfo>("sync_pairing_start");
}

export function cancelPairing() {
  return invoke<void>("sync_pairing_cancel");
}

export function revokeDevice(deviceId: string) {
  return invoke<void>("sync_revoke_device", { deviceId });
}
