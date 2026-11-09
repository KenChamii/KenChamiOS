import type { CSSProperties } from "react";

export const COUNT_UP_MS = 250;

/** The number shown `elapsed` ms into counting from `from` to `to`, easing out. */
export function countUpValue(from: number, to: number, elapsed: number, duration: number): number {
  const progress = duration <= 0 ? 1 : Math.min(Math.max(elapsed / duration, 0), 1);
  const eased = 1 - (1 - progress) ** 3;
  return Math.round(from + (to - from) * eased);
}

/** Position in a list for the `stagger-in` animation. */
export function stagger(index: number): CSSProperties {
  return { "--i": index } as CSSProperties;
}
