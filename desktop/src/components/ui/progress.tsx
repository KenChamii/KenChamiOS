export function Progress({ value, max }: { value: number; max: number }) {
  const percent = max === 0 ? 0 : Math.round((value / max) * 100);
  return (
    <div
      role="progressbar"
      aria-valuenow={value}
      aria-valuemax={max}
      className="h-1.5 w-full overflow-hidden rounded-full bg-muted"
    >
      <div className="h-full bg-primary" style={{ width: `${percent}%` }} />
    </div>
  );
}
