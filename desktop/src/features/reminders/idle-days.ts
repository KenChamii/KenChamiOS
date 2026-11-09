export const MIN_IDLE_DAYS = 1;
export const MAX_IDLE_DAYS = 365;

/** The number of days typed in the settings field, or null when it is not a whole number in range. */
export function parseIdleDays(text: string): number | null {
  if (!/^\d+$/.test(text.trim())) return null;
  const days = Number(text);
  return days >= MIN_IDLE_DAYS && days <= MAX_IDLE_DAYS ? days : null;
}

export function idleLabel(days: number): string {
  if (days === 0) return "Quiet since today";
  return days === 1 ? "Quiet for 1 day" : `Quiet for ${days} days`;
}
