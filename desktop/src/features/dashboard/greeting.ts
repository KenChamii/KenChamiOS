/** "Good morning" from 5:00, "Good afternoon" from 12:00, "Good evening" from 18:00. */
export function greetingFor(hour: number): string {
  if (hour >= 5 && hour < 12) return "Good morning";
  if (hour >= 12 && hour < 18) return "Good afternoon";
  return "Good evening";
}

/** "Tuesday, 10 November" in the user's locale. */
export function formatToday(ms: number): string {
  return new Date(ms).toLocaleDateString(undefined, {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
}
