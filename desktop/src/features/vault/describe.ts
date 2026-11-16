import type { VaultChange } from "@/lib/api/vault";

/** A short phrase for one planned or done change: where it goes and what it is. */
export function describeChange(change: VaultChange): string {
  if (change.kind === "conflict") return "Conflict, vault wins";
  const where = change.direction === "to_vault" ? "to vault" : "to app";
  return `${change.kind === "added" ? "New" : "Updated"} ${where}`;
}
