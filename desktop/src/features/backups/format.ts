const MAX_KEEP = 365;

/** How many backups to keep, as typed; null unless it is a whole number from 1 to 365. */
export function parseKeep(text: string): number | null {
  if (!/^\d+$/.test(text.trim())) return null;
  const keep = Number(text);
  return keep >= 1 && keep <= MAX_KEEP ? keep : null;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
