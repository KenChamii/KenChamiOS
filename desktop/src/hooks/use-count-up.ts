import { useEffect, useRef, useState } from "react";
import { COUNT_UP_MS, countUpValue } from "@/lib/motion";

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** Counts up to `target` once it is known, and from the old value when it changes. */
export function useCountUp(target: number): number {
  const [shown, setShown] = useState(0);
  const shownRef = useRef(0);

  useEffect(() => {
    const from = shownRef.current;
    const duration = reducedMotion.matches ? 0 : COUNT_UP_MS;
    const start = performance.now();
    let frame = 0;
    function step(now: number) {
      const value = countUpValue(from, target, now - start, duration);
      shownRef.current = value;
      setShown(value);
      if (value !== target) frame = requestAnimationFrame(step);
    }
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [target]);

  return shown;
}
