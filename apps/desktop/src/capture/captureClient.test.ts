import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  listMicrophones,
  subscribeToCapture,
  toCaptureError,
  toReplyEvent,
  toTranscriptEvent,
  toTranslationEvent,
} from "./captureClient";

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

describe("toCaptureError", () => {
  it("keeps a known code and its source", () => {
    expect(toCaptureError({ code: "permission_denied", audioSource: "system" })).toEqual({
      code: "permission_denied",
      audioSource: "system",
    });
  });

  it("drops fields it does not understand", () => {
    expect(toCaptureError({ code: "interrupted", audioSource: "radio", detail: "x" })).toEqual({
      code: "interrupted",
      audioSource: null,
    });
  });

  it("treats anything else as unknown", () => {
    for (const value of [null, "boom", 7, { code: "made_up" }, new Error("x")]) {
      expect(toCaptureError(value)).toEqual({ code: "unknown", audioSource: null });
    }
  });
});

describe("listMicrophones", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("returns the microphones from the core", async () => {
    const microphone = { id: "mic-1", name: "Desk microphone", isDefault: true };
    invoke.mockResolvedValue([microphone]);
    await expect(listMicrophones()).resolves.toEqual([microphone]);
    expect(invoke).toHaveBeenCalledWith("list_microphones");
  });

  it("ignores malformed entries and replies", async () => {
    invoke.mockResolvedValue([{ id: 1 }, "junk", { id: "a", name: "A", isDefault: false }]);
    await expect(listMicrophones()).resolves.toEqual([{ id: "a", name: "A", isDefault: false }]);
    invoke.mockResolvedValue("not a list");
    await expect(listMicrophones()).resolves.toEqual([]);
  });
});

describe("subscribeToCapture", () => {
  it("delivers validated events and unsubscribes from both", async () => {
    const callbacks = new Map<string, (event: { payload: unknown }) => void>();
    const unlisten = vi.fn();
    listen.mockImplementation((name: string, callback: (event: { payload: unknown }) => void) => {
      callbacks.set(name, callback);
      return Promise.resolve(unlisten);
    });
    const onLevels = vi.fn();
    const onInterrupted = vi.fn();
    const onTranscript = vi.fn();
    const onTranslation = vi.fn();
    const onReplies = vi.fn();

    const unsubscribe = await subscribeToCapture({
      onLevels,
      onInterrupted,
      onTranscript,
      onTranslation,
      onReplies,
    });
    callbacks.get("reply-event")?.({ payload: { status: "pending" } });
    callbacks.get("translation-event")?.({ payload: { segmentId: "a", status: "pending" } });
    callbacks.get("transcript-event")?.({ payload: { kind: "discarded", id: "microphone-1-0" } });
    callbacks.get("transcript-event")?.({ payload: { kind: "nonsense" } });
    callbacks.get("capture-levels")?.({
      payload: { microphone: { peak: 0.5, rms: 0.1 }, system: "bad", droppedFrames: 3 },
    });
    callbacks.get("capture-interrupted")?.({
      payload: { code: "interrupted", audioSource: "microphone" },
    });

    expect(onLevels).toHaveBeenCalledWith({
      microphone: { peak: 0.5, rms: 0.1 },
      system: null,
      droppedFrames: 3,
    });
    expect(onInterrupted).toHaveBeenCalledWith({ code: "interrupted", audioSource: "microphone" });

    // The malformed transcript event was dropped.
    expect(onTranscript).toHaveBeenCalledExactlyOnceWith({
      kind: "discarded",
      id: "microphone-1-0",
    });

    expect(onTranslation).toHaveBeenCalledExactlyOnceWith({
      segmentId: "a",
      state: { status: "pending" },
    });

    expect(onReplies).toHaveBeenCalledExactlyOnceWith({ status: "pending" });

    unsubscribe();
    expect(unlisten).toHaveBeenCalledTimes(5);
  });
});

describe("toTranscriptEvent", () => {
  const segment = {
    id: "system-1-0",
    source: "system",
    startMs: 1200,
    text: "Hello",
    language: null,
    isFinal: false,
  };

  it("accepts a well-formed segment", () => {
    expect(toTranscriptEvent({ kind: "segment", segment })).toEqual({ kind: "segment", segment });
  });

  it("rejects a segment with a missing or wrong field", () => {
    expect(
      toTranscriptEvent({ kind: "segment", segment: { ...segment, source: "radio" } }),
    ).toBeNull();
    expect(toTranscriptEvent({ kind: "segment", segment: { ...segment, text: 5 } })).toBeNull();
    expect(toTranscriptEvent({ kind: "segment" })).toBeNull();
  });

  it("reads a failure as a typed error", () => {
    const event = { kind: "failed", source: "microphone", error: { code: "rate_limited" } };
    expect(toTranscriptEvent(event)).toEqual({
      kind: "failed",
      error: { code: "rate_limited", audioSource: null },
    });
  });
});

describe("toTranslationEvent", () => {
  it("reads a finished translation", () => {
    const event = {
      segmentId: "system-1-0",
      status: "done",
      translation: { text: "Salam.", targetLanguage: "az", unpreserved: ["14"] },
    };
    expect(toTranslationEvent(event)).toEqual({
      segmentId: "system-1-0",
      state: { status: "done", text: "Salam.", targetLanguage: "az", unpreserved: ["14"] },
    });
  });

  it("reads a failure and falls back to unknown for a new error code", () => {
    const failed = (code: string) => ({ segmentId: "a", status: "failed", error: { code } });
    expect(toTranslationEvent(failed("rate_limited"))?.state).toEqual({
      status: "failed",
      reason: "rate_limited",
    });
    expect(toTranslationEvent(failed("brand_new"))?.state).toEqual({
      status: "failed",
      reason: "unknown",
    });
  });

  it("rejects malformed events", () => {
    expect(toTranslationEvent({ status: "pending" })).toBeNull();
    expect(toTranslationEvent({ segmentId: "a", status: "done", translation: {} })).toBeNull();
    expect(toTranslationEvent({ segmentId: "a", status: "nonsense" })).toBeNull();
  });
});

describe("toReplyEvent", () => {
  const suggestion = {
    style: "concise",
    text: "Understood.",
    meaning: null,
    asksForMissingFacts: false,
  };

  it("reads ready suggestions and drops malformed ones", () => {
    const event = { status: "ready", suggestions: [suggestion, { style: "rude", text: "x" }] };
    expect(toReplyEvent(event)).toEqual({ status: "ready", suggestions: [suggestion] });
  });

  it("reads a failure reason", () => {
    expect(toReplyEvent({ status: "failed", error: { code: "refused" } })).toEqual({
      status: "failed",
      reason: "refused",
    });
  });

  it("rejects an unknown status", () => {
    expect(toReplyEvent({ status: "sent" })).toBeNull();
    expect(toReplyEvent(null)).toBeNull();
  });
});
