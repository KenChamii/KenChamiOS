import type { PairingPayload } from "@/lib/api/sync";

/** The pairing data as text, for typing or pasting on the phone. */
export function pairingText(payload: PairingPayload): string {
  return JSON.stringify(payload);
}

/** "4:59" style countdown; never negative. */
export function countdown(seconds: number): string {
  const left = Math.max(0, Math.floor(seconds));
  const minutes = Math.floor(left / 60);
  return `${minutes}:${String(left % 60).padStart(2, "0")}`;
}

/** Groups the fingerprint in blocks of 8 characters so it can be compared by eye. */
export function groupFingerprint(fingerprint: string): string {
  return fingerprint.match(/.{1,8}/g)?.join(" ") ?? fingerprint;
}
