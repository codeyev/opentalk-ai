import { useCallback, useEffect, useState } from "react";
import {
  NO_REPLIES,
  type Replies,
  type SessionState,
  type TranscriptSegment,
  type Translations,
} from "../domain/conversation";
import {
  type CaptureConfig,
  type CaptureError,
  isCapturing,
  type LevelsUpdate,
  listMicrophones,
  type MicrophoneInfo,
  refreshReplies,
  type ReplyEvent,
  type SessionFeatures,
  startCapture,
  stopCapture,
  subscribeToCapture,
  toCaptureError,
} from "./captureClient";
import { applyTranscriptEvent } from "./transcript";

interface Session {
  state: SessionState;
  /** Start was pressed and the core has not answered yet. Loading a model can take seconds. */
  isStarting: boolean;
  /** When the current session started, as a `Date.now()` value; `null` when not capturing. */
  startedAt: number | null;
  /** Why the last session failed or stopped by itself. */
  error: CaptureError | null;
  /** Latest input levels; `null` when not capturing. */
  levels: LevelsUpdate | null;
  /** The transcript of the current or most recent session, oldest first. */
  segments: readonly TranscriptSegment[];
  /** Translation progress for the segments above. */
  translations: Translations;
  replies: Replies;
  /** Why transcription stopped while capture carried on. */
  transcriptionError: CaptureError | null;
}

export interface Capture extends Session {
  microphones: MicrophoneInfo[];
  start: (config: CaptureConfig, features: SessionFeatures) => void;
  stop: () => void;
  /** Asks for fresh reply suggestions. */
  refreshReplies: () => void;
}

const IDLE: Session = {
  state: "idle",
  isStarting: false,
  startedAt: null,
  error: null,
  levels: null,
  segments: [],
  translations: {},
  replies: NO_REPLIES,
  transcriptionError: null,
};

/** A session that is capturing, starting now. */
function listening(): Session {
  return { ...IDLE, state: "listening", startedAt: Date.now() };
}

/** The session after capture ended; what was transcribed stays on screen. */
function stopped(session: Session, state: SessionState, error: CaptureError | null): Session {
  return { ...session, state, error, isStarting: false, startedAt: null, levels: null };
}

/** Applies a reply event. Suggestions stay listed until newer ones replace them. */
function applyReplyEvent(replies: Replies, event: ReplyEvent): Replies {
  switch (event.status) {
    case "pending":
      return { ...replies, status: "pending", failure: null };
    case "ready":
      return {
        status: "ready",
        suggestions: event.suggestions,
        failure: null,
        generation: replies.generation + 1,
      };
    case "failed":
      return { ...replies, status: "failed", failure: event.reason };
  }
}

/** Owns the capture session's UI state and keeps it in step with the core. */
export function useCapture(): Capture {
  const [session, setSession] = useState<Session>(IDLE);
  const [microphones, setMicrophones] = useState<MicrophoneInfo[]>([]);

  useEffect(() => {
    let isCurrent = true;
    // Without the Tauri shell (tests, a plain browser) the list stays empty.
    listMicrophones().then(
      (found) => {
        if (isCurrent) {
          setMicrophones(found);
        }
      },
      () => undefined,
    );
    return () => {
      isCurrent = false;
    };
  }, []);

  useEffect(() => {
    let isCurrent = true;
    let unsubscribe: (() => void) | undefined;
    subscribeToCapture({
      onLevels: (levels) => {
        setSession((current) => (current.state === "listening" ? { ...current, levels } : current));
      },
      onInterrupted: (error) => {
        setSession((current) => stopped(current, "error", error));
      },
      onTranscript: (event) => {
        setSession((current) =>
          event.kind === "failed"
            ? { ...current, transcriptionError: event.error }
            : { ...current, segments: applyTranscriptEvent(current.segments, event) },
        );
      },
      onTranslation: ({ segmentId, state }) => {
        setSession((current) => ({
          ...current,
          translations: { ...current.translations, [segmentId]: state },
        }));
      },
      onReplies: (event) => {
        setSession((current) => ({ ...current, replies: applyReplyEvent(current.replies, event) }));
      },
    }).then(
      (stopListening) => {
        if (isCurrent) {
          unsubscribe = stopListening;
        } else {
          stopListening();
        }
      },
      () => undefined,
    );
    return () => {
      isCurrent = false;
      unsubscribe?.();
    };
  }, []);

  // The core may already be capturing when the interface loads, for example
  // after the page was reloaded. Showing "idle" then would hide a live capture.
  useEffect(() => {
    let isCurrent = true;
    isCapturing().then(
      (isRunning) => {
        if (isCurrent && isRunning) {
          setSession((current) => (current.state === "listening" ? current : listening()));
        }
      },
      () => undefined,
    );
    return () => {
      isCurrent = false;
    };
  }, []);

  const start = useCallback((config: CaptureConfig, features: SessionFeatures) => {
    setSession((current) => ({ ...current, isStarting: true, error: null }));
    startCapture(config, features).then(
      () => {
        setSession(listening());
      },
      (failure: unknown) => {
        const error = toCaptureError(failure);
        setSession((current) => {
          // "Already running" means audio is being captured. The interface
          // must show that, with its Stop button, never an idle screen.
          if (error.code === "already_running") {
            return current.state === "listening" ? { ...current, isStarting: false } : listening();
          }
          return { ...IDLE, state: "error", error };
        });
      },
    );
  }, []);

  const stop = useCallback(() => {
    // The UI goes idle either way: if the core cannot be reached, nothing is capturing.
    const goIdle = () => {
      setSession((current) => stopped(current, "idle", null));
    };
    stopCapture().then(goIdle, goIdle);
  }, []);

  const refresh = useCallback(() => {
    // A failed request changes nothing: the suggestions on screen simply stay.
    refreshReplies().catch(() => undefined);
  }, []);

  return { ...session, microphones, start, stop, refreshReplies: refresh };
}
