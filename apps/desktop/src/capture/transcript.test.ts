import { describe, expect, it } from "vitest";
import type { TranscriptSegment } from "../domain/conversation";
import { applyTranscriptEvent, MAX_SEGMENTS } from "./transcript";

function segment(id: string, startMs: number, text = id, isFinal = true): TranscriptSegment {
  return { id, source: "microphone", startMs, text, language: null, isFinal };
}

describe("applyTranscriptEvent", () => {
  it("adds a new segment", () => {
    const result = applyTranscriptEvent([], { kind: "segment", segment: segment("a", 0) });
    expect(result).toEqual([segment("a", 0)]);
  });

  it("replaces a partial segment with its final version", () => {
    const partial = segment("a", 0, "Hel", false);
    const final = segment("a", 0, "Hello.", true);
    expect(applyTranscriptEvent([partial], { kind: "segment", segment: final })).toEqual([final]);
  });

  it("keeps segments from different sources in order of start time", () => {
    const later = segment("later", 5000);
    const earlier = segment("earlier", 1000);
    const result = applyTranscriptEvent([later], { kind: "segment", segment: earlier });
    expect(result.map((item) => item.id)).toEqual(["earlier", "later"]);
  });

  it("removes a discarded segment", () => {
    const segments = [segment("a", 0), segment("b", 1000)];
    expect(applyTranscriptEvent(segments, { kind: "discarded", id: "a" })).toEqual([
      segment("b", 1000),
    ]);
  });

  it("ignores a discard for an unknown segment", () => {
    const segments = [segment("a", 0)];
    expect(applyTranscriptEvent(segments, { kind: "discarded", id: "zzz" })).toEqual(segments);
  });

  it("drops the oldest segments beyond the limit", () => {
    const full = Array.from({ length: MAX_SEGMENTS }, (_, index) =>
      segment(`s${String(index)}`, index),
    );
    const result = applyTranscriptEvent(full, {
      kind: "segment",
      segment: segment("newest", MAX_SEGMENTS),
    });
    expect(result).toHaveLength(MAX_SEGMENTS);
    expect(result[0]?.id).toBe("s1");
    expect(result.at(-1)?.id).toBe("newest");
  });
});
