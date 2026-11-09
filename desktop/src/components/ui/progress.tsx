export function Progress({ value, max }: { value: number; max: number }) {
  const percent = max === 0 ? 0 : Math.round((value / max) * 100);
  return (
    <div
      role="progressbar"
      aria-valuenow={value}
      aria-valuemax={max}
      className="h-2 w-full overflow-hidden rounded-full bg-muted"
    >
      <div
        className="h-full bg-accent transition-[width] duration-(--duration-base) ease-(--ease-out)"
        style={{ width: `${percent}%` }}
      />
    </div>
  );
}
