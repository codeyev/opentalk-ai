import type { ReactNode } from "react";
import { CaptureContext } from "./CaptureContext";
import { useCapture } from "./useCapture";

/**
 * Holds the capture session above the screens, so it keeps its state while
 * the user visits settings and comes back.
 */
export function CaptureProvider({ children }: { children: ReactNode }) {
  const capture = useCapture();
  return <CaptureContext value={capture}>{children}</CaptureContext>;
}
