import type { TranscriptSegment } from "../domain/conversation";
import type { TranscriptEvent } from "./captureClient";

/** Most segments kept on screen; older ones are dropped so memory stays bounded. */
export const MAX_SEGMENTS = 500;

type SegmentChange = Exclude<TranscriptEvent, { kind: "failed" }>;

/**
 * Applies a transcript event to the list of segments, keeping it ordered by
 * start time. A segment event replaces the segment with the same id (a
 * partial becoming final) or adds a new one.
 */
export function applyTranscriptEvent(
  segments: readonly TranscriptSegment[],
  event: SegmentChange,
): readonly TranscriptSegment[] {
  const id = event.kind === "segment" ? event.segment.id : event.id;
  const others = segments.filter((segment) => segment.id !== id);
  if (event.kind === "discarded") {
    return others;
  }
  return [...others, event.segment]
    .sort((first, second) => first.startMs - second.startMs)
    .slice(-MAX_SEGMENTS);
}
