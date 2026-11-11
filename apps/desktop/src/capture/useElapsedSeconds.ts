import { useEffect, useState } from "react";

/** Whole seconds since `startedAt`, updated once a second; `null` when not running. */
export function useElapsedSeconds(startedAt: number | null): number | null {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (startedAt === null) {
      return;
    }
    const timer = window.setInterval(() => {
      setNow(Date.now());
    }, 1000);
    return () => {
      window.clearInterval(timer);
    };
  }, [startedAt]);

  return startedAt === null ? null : Math.max(0, Math.floor((now - startedAt) / 1000));
}
