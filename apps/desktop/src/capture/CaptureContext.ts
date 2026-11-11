import { createContext, useContext } from "react";
import type { Capture } from "./useCapture";

export const CaptureContext = createContext<Capture | null>(null);

/** The app's one capture session. It outlives navigation between screens. */
export function useCaptureSession(): Capture {
  const capture = useContext(CaptureContext);
  if (capture === null) {
    throw new Error("useCaptureSession must be used inside a CaptureProvider.");
  }
  return capture;
}
