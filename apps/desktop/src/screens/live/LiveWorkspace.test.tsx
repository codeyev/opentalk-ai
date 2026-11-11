import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DEMO_CONVERSATION } from "../../demo/demoConversation";
import { fakeCapture, fakeCaptureClientModule } from "../../test/fakeCapture";
import { fakeProvider, fakeProviderClientModule } from "../../test/fakeProvider";
import { renderApp } from "../../test/renderApp";

vi.mock("../../capture/captureClient", () => fakeCaptureClientModule());
vi.mock("../../providers/providerClient", () => fakeProviderClientModule());

/** Turns transcription on in settings, as the Providers screen would, with a key saved. */
async function enableTranscription() {
  fireEvent.click(screen.getByRole("button", { name: "Providers" }));
  fireEvent.click(await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }));
  fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
  await screen.findByText(/Transcription is on/);
}

/** Turns transcription and translation on, with a key saved. */
async function enableTranslation() {
  fireEvent.click(screen.getByRole("button", { name: "Providers" }));
  fireEvent.click(await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }));
  fireEvent.click(screen.getByRole("checkbox", { name: /Send transcript text to OpenAI/ }));
  fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
  await screen.findByText(/Translation into English is on/);
}

/** Translations are shown under their segment, inside the conversation. */
function translation() {
  return conversation();
}

function conversation() {
  return screen.getByRole("region", { name: /^Conversation/ });
}

const writeText = vi.fn();

/** The Arabic style sent with a request when the user has not changed the defaults. */
const DEFAULT_ARABIC = { dialect: null, tashkeel: "off" };

/** A session that only captures: nothing is sent to any provider. */
const NO_FEATURES = { localOnly: false, transcription: null, translation: null, replies: null };

function showDemoData() {
  fireEvent.click(screen.getByRole("checkbox", { name: "Show demo data" }));
}

async function startCapture() {
  fireEvent.click(screen.getByRole("button", { name: "Start" }));
  await screen.findByText("Capturing audio");
}

describe("Live workspace", () => {
  beforeEach(() => {
    fakeCapture.reset();
    fakeProvider.reset();
    writeText.mockReset();
    writeText.mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  });

  describe("audio capture", () => {
    it("is idle until the user presses Start", () => {
      renderApp();
      expect(screen.getByText("Idle")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Start" })).toBeEnabled();
      expect(screen.queryByRole("button", { name: "Stop" })).not.toBeInTheDocument();
      expect(screen.queryByText("Capturing audio")).not.toBeInTheDocument();
      expect(fakeCapture.startCapture).not.toHaveBeenCalled();
    });

    it("lists the microphones reported by the core", async () => {
      fakeCapture.microphones = [{ id: "mic-1", name: "Desk microphone", isDefault: true }];
      renderApp();
      expect(await screen.findByRole("option", { name: "Desk microphone" })).toBeInTheDocument();
      expect(screen.getByRole("option", { name: "System default" })).toBeInTheDocument();
    });

    it("starts capture with the selected sources", async () => {
      fakeCapture.microphones = [{ id: "mic-1", name: "Desk microphone", isDefault: true }];
      renderApp();
      await screen.findByRole("option", { name: "Desk microphone" });
      fireEvent.change(screen.getByRole("combobox", { name: "Microphone" }), {
        target: { value: "mic-1" },
      });
      fireEvent.click(screen.getByRole("checkbox", { name: "Capture system audio" }));
      await startCapture();

      expect(fakeCapture.startCapture).toHaveBeenCalledWith(
        { microphone: true, microphoneId: "mic-1", systemAudio: true },
        NO_FEATURES,
      );
      expect(screen.getByText("Listening")).toBeInTheDocument();
    });

    it("locks the source controls while capturing and releases them on Stop", async () => {
      renderApp();
      await startCapture();
      expect(screen.queryByRole("button", { name: "Start" })).not.toBeInTheDocument();
      expect(screen.getByRole("checkbox", { name: "Capture microphone" })).toBeDisabled();
      expect(screen.getByRole("checkbox", { name: "Capture system audio" })).toBeDisabled();

      fireEvent.click(screen.getByRole("button", { name: "Stop" }));
      await waitFor(() => {
        expect(screen.getByText("Idle")).toBeInTheDocument();
      });
      expect(fakeCapture.stopCapture).toHaveBeenCalledOnce();
      expect(screen.queryByText("Capturing audio")).not.toBeInTheDocument();
    });

    it("shows that it is starting, and cannot be started twice", async () => {
      // The core takes a while to answer, as it does when it loads a model.
      let finishStarting: () => void = () => undefined;
      fakeCapture.startCapture.mockReturnValue(
        new Promise((resolve) => {
          finishStarting = resolve;
        }),
      );
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));

      const starting = screen.getByRole("button", { name: "Starting…" });
      expect(starting).toBeDisabled();
      expect(screen.getByText(/Loading a local model can take a few seconds/)).toBeInTheDocument();
      fireEvent.click(starting);
      expect(fakeCapture.startCapture).toHaveBeenCalledOnce();

      act(() => {
        finishStarting();
      });
      expect(await screen.findByText("Capturing audio")).toBeInTheDocument();
    });

    it("shows a running capture, with Stop, when the core says it is already running", async () => {
      fakeCapture.startCapture.mockRejectedValue({ code: "already_running" });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));

      // Audio is being captured, so the interface must never look idle.
      expect(await screen.findByText("Capturing audio")).toBeInTheDocument();
      expect(screen.getByText("Listening")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    });

    it("shows a capture that was already running when the interface loaded", async () => {
      fakeCapture.isCapturing = true;
      renderApp();
      expect(await screen.findByText("Capturing audio")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
    });

    it("keeps the session when the user visits settings and comes back", async () => {
      renderApp();
      await startCapture();
      fireEvent.click(screen.getByRole("button", { name: "About" }));
      // The sidebar shows the session on every screen.
      expect(screen.getByText("Listening")).toBeInTheDocument();

      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(screen.getByText("Capturing audio")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
    });

    it("cannot start without an audio source", () => {
      renderApp();
      fireEvent.click(screen.getByRole("checkbox", { name: "Capture microphone" }));
      expect(screen.getByRole("button", { name: "Start" })).toBeDisabled();
    });

    it("shows a level meter for each captured source", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onLevels({
          microphone: { peak: 0.4, rms: 0.2 },
          system: null,
          droppedFrames: 0,
        });
      });
      expect(screen.getByRole("meter", { name: "Microphone level" })).toHaveValue(0.4);
      expect(screen.queryByRole("meter", { name: "System audio level" })).not.toBeInTheDocument();
    });

    it("explains a denied permission and stays stopped", async () => {
      fakeCapture.startCapture.mockRejectedValue({
        code: "permission_denied",
        audioSource: "system",
      });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));

      expect(await screen.findByRole("alert")).toHaveTextContent(/System audio access was denied/);
      expect(screen.getByText("Error")).toBeInTheDocument();
      expect(screen.queryByText("Capturing audio")).not.toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Start" })).toBeEnabled();
    });

    it("stops and reports when capture is interrupted", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onInterrupted({ code: "interrupted", audioSource: "microphone" });
      });
      expect(screen.getByRole("alert")).toHaveTextContent(/Capture stopped unexpectedly/);
      expect(screen.queryByText("Capturing audio")).not.toBeInTheDocument();
    });

    it("falls back to a general message for an unrecognised error", async () => {
      fakeCapture.startCapture.mockRejectedValue("boom");
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));
      expect(await screen.findByRole("alert")).toHaveTextContent(
        "Something went wrong with audio capture.",
      );
    });
  });

  describe("transcription", () => {
    const segment = {
      id: "system-1-0",
      source: "system" as const,
      startMs: 4000,
      text: "Hello the",
      language: "en",
      isFinal: false,
    };

    it("is off by default, so no speech leaves the computer", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      expect(await screen.findByText(/Transcription is off/)).toBeInTheDocument();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), NO_FEATURES);
    });

    it("stays off without a saved key even if it was enabled earlier", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      await enableTranscription();
      fakeProvider.reset({ hasApiKey: false });

      // Reopening the screen reloads the provider's key status.
      fireEvent.click(screen.getByRole("button", { name: "About" }));
      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(await screen.findByText(/Transcription is off/)).toBeInTheDocument();
    });

    it("transcribes on this computer with Whisper, without an API key", async () => {
      fakeProvider.whisper = {
        ...fakeProvider.whisper,
        installed: [{ fileName: "ggml-base.bin", sizeBytes: 147_951_465 }],
      };
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Providers" }));
      const panel = within(screen.getByRole("region", { name: "Transcription" }));
      fireEvent.click(await panel.findByRole("radio", { name: "Whisper · on this computer" }));
      fireEvent.change(await screen.findByLabelText("Model"), {
        target: { value: "ggml-base.bin" },
      });
      fireEvent.click(
        panel.getByRole("checkbox", { name: /Transcribe captured speech with Whisper/ }),
      );

      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(
        await screen.findByText(/done by Whisper on this computer. No audio leaves it/),
      ).toBeInTheDocument();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), {
        ...NO_FEATURES,
        transcription: { provider: { id: "whisper", model: "ggml-base.bin" }, language: null },
      });
    });

    it("requests transcription in the chosen spoken language when enabled", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      await enableTranscription();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), {
        ...NO_FEATURES,
        transcription: { provider: { id: "openai" }, language: null },
      });
    });

    it("shows partial text, then the final text in its place", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onTranscript({ kind: "segment", segment });
      });
      expect(within(conversation()).getByText(/Hello the/)).toBeInTheDocument();
      expect(within(conversation()).getByText("In progress")).toBeInTheDocument();
      expect(within(conversation()).getByText("System audio")).toBeInTheDocument();
      expect(within(conversation()).getByText("00:04")).toBeInTheDocument();

      act(() => {
        fakeCapture.events().onTranscript({
          kind: "segment",
          segment: { ...segment, text: "Hello there.", isFinal: true },
        });
      });
      expect(within(conversation()).getByText("Hello there.")).toBeInTheDocument();
      expect(within(conversation()).queryByText("In progress")).not.toBeInTheDocument();
      expect(within(conversation()).getAllByRole("listitem")).toHaveLength(1);
    });

    it("removes a segment that turned out not to be speech", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onTranscript({ kind: "segment", segment });
        fakeCapture.events().onTranscript({ kind: "discarded", id: segment.id });
      });
      expect(within(conversation()).getByText(/No transcript yet/)).toBeInTheDocument();
    });

    it("keeps the transcript on screen after Stop and clears it on the next Start", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onTranscript({ kind: "segment", segment });
      });
      fireEvent.click(screen.getByRole("button", { name: "Stop" }));
      await waitFor(() => {
        expect(screen.getByText("Idle")).toBeInTheDocument();
      });
      expect(within(conversation()).getByText(/Hello the/)).toBeInTheDocument();

      await startCapture();
      expect(within(conversation()).getByText(/No transcript yet/)).toBeInTheDocument();
    });

    it("reports a transcription failure while capture carries on", async () => {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onTranscript({
          kind: "failed",
          error: { code: "authentication_failed", audioSource: null },
        });
      });
      expect(screen.getByRole("alert")).toHaveTextContent(
        /rejected the API key.*Audio capture continues/,
      );
      expect(screen.getByText("Capturing audio")).toBeInTheDocument();
    });

    it("explains a missing key when capture cannot start", async () => {
      fakeCapture.startCapture.mockRejectedValue({ code: "missing_api_key" });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));
      expect(await screen.findByRole("alert")).toHaveTextContent(/No API key is saved/);
    });
  });

  describe("translation", () => {
    const finalSegment = {
      id: "system-1-0",
      source: "system" as const,
      startMs: 4000,
      text: "It costs 1,200 USD.",
      language: "en",
      isFinal: true,
    };

    async function captureWithSegment() {
      renderApp();
      await startCapture();
      act(() => {
        fakeCapture.events().onTranscript({ kind: "segment", segment: finalSegment });
      });
    }

    function report(
      state: Parameters<ReturnType<typeof fakeCapture.events>["onTranslation"]>[0]["state"],
    ) {
      act(() => {
        fakeCapture.events().onTranslation({ segmentId: finalSegment.id, state });
      });
    }

    it("is requested with the target language and glossary when turned on", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      await enableTranslation();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), {
        localOnly: false,
        transcription: { provider: { id: "openai" }, language: null },
        translation: {
          provider: { id: "openai" },
          targetLanguage: "en",
          glossary: [],
          arabic: DEFAULT_ARABIC,
        },
        replies: null,
      });
    });

    it("uses the local model when Ollama is chosen, and says nothing leaves the computer", async () => {
      fakeProvider.reset({ hasApiKey: true });
      fakeProvider.ollama = { version: "0.40.14", models: [{ name: "qwen3.5:9b", sizeBytes: 1 }] };
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Providers" }));
      fireEvent.click(
        await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }),
      );
      fireEvent.change(await screen.findByLabelText("Model"), { target: { value: "qwen3.5:9b" } });
      const panel = screen.getByRole("region", { name: "Translation" });
      fireEvent.click(within(panel).getByRole("radio", { name: /Ollama · on this computer/ }));
      fireEvent.click(screen.getByRole("checkbox", { name: /Send transcript text to Ollama/ }));

      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(
        await screen.findByText(/Translation into English is on, done by Ollama on this computer/),
      ).toBeInTheDocument();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(
        expect.anything(),
        expect.objectContaining({
          translation: expect.objectContaining({
            provider: { id: "ollama", endpoint: "http://localhost:11434", model: "qwen3.5:9b" },
          }) as unknown,
        }),
      );
    });

    it("uses the user's own service when one is chosen", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Providers" }));
      fireEvent.click(
        await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }),
      );
      const custom = within(screen.getByRole("region", { name: "Other AI service" }));
      fireEvent.change(custom.getByLabelText("Model name"), {
        target: { value: "openai/gpt-4o-mini" },
      });
      fireEvent.change(custom.getByLabelText("API key"), { target: { value: "sk-or-key" } });
      fireEvent.click(custom.getByRole("button", { name: "Save key" }));
      await custom.findByText(/An API key is saved/);
      const panel = within(screen.getByRole("region", { name: "Translation" }));
      fireEvent.click(panel.getByRole("radio", { name: "openrouter.ai · cloud service" }));
      fireEvent.click(
        panel.getByRole("checkbox", { name: /Send transcript text to openrouter.ai/ }),
      );

      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(
        await screen.findByText(/finished transcript text is sent to openrouter.ai/),
      ).toBeInTheDocument();
      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(
        expect.anything(),
        expect.objectContaining({
          translation: expect.objectContaining({
            provider: {
              id: "custom",
              baseUrl: "https://openrouter.ai/api/v1",
              model: "openai/gpt-4o-mini",
            },
          }) as unknown,
        }),
      );
    });

    it("sends nothing to a cloud service in local-only mode", async () => {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      await enableTranslation();
      fireEvent.click(screen.getByRole("button", { name: "Providers" }));
      fireEvent.click(
        await screen.findByRole("checkbox", { name: "Never contact a cloud service" }),
      );
      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      expect(await screen.findByText(/Local-only mode is on/)).toBeInTheDocument();

      await startCapture();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), {
        ...NO_FEATURES,
        localOnly: true,
      });
    });

    it("explains when the core refuses a session because of local-only mode", async () => {
      fakeCapture.startCapture.mockRejectedValue({ code: "local_only" });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Start" }));
      expect(await screen.findByRole("alert")).toHaveTextContent(/Local-only mode is on/);
    });

    it("shows a segment as translating, then its translation", async () => {
      await captureWithSegment();
      report({ status: "pending" });
      expect(within(translation()).getByText("Translating…")).toBeInTheDocument();

      report({
        status: "done",
        text: "Qiyməti 1,200 USD-dir.",
        targetLanguage: "az",
        unpreserved: [],
      });
      expect(within(translation()).getByText("Qiyməti 1,200 USD-dir.")).toBeInTheDocument();
      expect(within(translation()).queryByText("Translating…")).not.toBeInTheDocument();
      expect(within(translation()).queryByText(/Check this translation/)).not.toBeInTheDocument();
    });

    it("asks the user to check a translation that lost a number", async () => {
      await captureWithSegment();
      report({
        status: "done",
        text: "Qiyməti 1,300 USD-dir.",
        targetLanguage: "az",
        unpreserved: ["1,200"],
      });
      expect(within(translation()).getByText(/Check this translation.*1,200/)).toBeInTheDocument();
    });

    it("says why a segment was not translated", async () => {
      await captureWithSegment();
      report({ status: "failed", reason: "rate_limited" });
      expect(
        within(translation()).getByText(/Not translated: the provider is limiting requests/),
      ).toBeInTheDocument();
      // The transcript itself is unaffected.
      expect(within(conversation()).getByText("It costs 1,200 USD.")).toBeInTheDocument();
    });

    it("labels diacritics in a translation as AI-generated", async () => {
      await captureWithSegment();
      report({ status: "done", text: "السعر", targetLanguage: "ar", unpreserved: [] });
      expect(within(translation()).queryByText(/added by AI/)).not.toBeInTheDocument();

      report({ status: "done", text: "السِّعْرُ", targetLanguage: "ar", unpreserved: [] });
      expect(
        within(translation()).getByText(/diacritics in this text were added by AI/),
      ).toBeInTheDocument();
    });

    it("lets the user copy a translation", async () => {
      await captureWithSegment();
      report({ status: "done", text: "السعر 1,200 USD.", targetLanguage: "ar", unpreserved: [] });
      fireEvent.click(within(translation()).getByRole("button", { name: "Copy" }));
      // Mixed Arabic, Latin and digits are copied exactly as stored, in logical order.
      expect(writeText).toHaveBeenCalledWith("السعر 1,200 USD.");
    });

    it("lays out an Arabic translation as Arabic text", async () => {
      await captureWithSegment();
      report({ status: "done", text: "السعر 1,200 دولار.", targetLanguage: "ar", unpreserved: [] });
      const text = within(translation()).getByText("السعر 1,200 دولار.");
      expect(text).toHaveAttribute("lang", "ar");
      expect(text).toHaveAttribute("dir", "auto");
    });
  });

  describe("reply suggestions", () => {
    const suggestions = [
      {
        style: "professional" as const,
        text: "Could you share the feature list?",
        meaning: "Funksiyaların siyahısını paylaşa bilərsiniz?",
        asksForMissingFacts: true,
      },
      { style: "concise" as const, text: "Understood.", meaning: null, asksForMissingFacts: false },
    ];

    function replies() {
      return screen.getByRole("region", { name: /^Reply suggestions/ });
    }

    function report(event: Parameters<ReturnType<typeof fakeCapture.events>["onReplies"]>[0]) {
      act(() => {
        fakeCapture.events().onReplies(event);
      });
    }

    /** Turns transcription and reply suggestions on, with a key saved, and starts capture. */
    async function captureWithReplies() {
      fakeProvider.reset({ hasApiKey: true });
      renderApp();
      fireEvent.click(screen.getByRole("button", { name: "Providers" }));
      fireEvent.click(
        await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }),
      );
      fireEvent.click(screen.getByRole("checkbox", { name: /Send the conversation to OpenAI/ }));
      fireEvent.click(screen.getByRole("button", { name: "Live workspace" }));
      await screen.findByText(/Reply suggestions are on/);
      await startCapture();
    }

    it("are requested with the reply language and business context when turned on", async () => {
      await captureWithReplies();
      expect(fakeCapture.startCapture).toHaveBeenCalledWith(expect.anything(), {
        localOnly: false,
        transcription: { provider: { id: "openai" }, language: null },
        translation: null,
        // Replies and the user's reading language are both English by default, so no meaning.
        replies: {
          provider: { id: "openai" },
          replyLanguage: "en",
          meaningLanguage: null,
          businessContext: "",
          arabic: DEFAULT_ARABIC,
        },
      });
    });

    it("cannot be refreshed unless they are on and capture is running", () => {
      renderApp();
      expect(screen.getByRole("button", { name: "Refresh suggestions" })).toBeDisabled();
    });

    it("shows suggestions with their meaning and a note about missing facts", async () => {
      await captureWithReplies();
      report({ status: "pending" });
      expect(within(replies()).getByText("Preparing new suggestions…")).toBeInTheDocument();

      report({ status: "ready", suggestions });
      expect(within(replies()).getAllByRole("article")).toHaveLength(2);
      expect(within(replies()).getByText("Could you share the feature list?")).toBeInTheDocument();
      expect(within(replies()).getByText(/Funksiyaların siyahısını/)).toBeInTheDocument();
      expect(within(replies()).getAllByText(/asks a question because a fact/)).toHaveLength(1);
      expect(within(replies()).queryByText("Preparing new suggestions…")).not.toBeInTheDocument();
    });

    it("marks suggestions as outdated while newer ones are prepared", async () => {
      await captureWithReplies();
      report({ status: "ready", suggestions });
      report({ status: "pending" });
      expect(within(replies()).getByText(/from earlier in the conversation/)).toBeInTheDocument();
      expect(within(replies()).getAllByRole("article")).toHaveLength(2);
    });

    it("asks the core for new suggestions on Refresh", async () => {
      await captureWithReplies();
      fireEvent.click(screen.getByRole("button", { name: "Refresh suggestions" }));
      expect(fakeCapture.refreshReplies).toHaveBeenCalledOnce();
    });

    it("dismisses a single suggestion, and shows it again in the next set", async () => {
      await captureWithReplies();
      report({ status: "ready", suggestions });
      fireEvent.click(screen.getByRole("button", { name: "Dismiss the Concise reply" }));
      expect(within(replies()).getAllByRole("article")).toHaveLength(1);
      expect(within(replies()).queryByText("Understood.")).not.toBeInTheDocument();

      report({ status: "ready", suggestions });
      expect(within(replies()).getAllByRole("article")).toHaveLength(2);
    });

    it("says why suggestions could not be prepared and keeps the old ones", async () => {
      await captureWithReplies();
      report({ status: "ready", suggestions });
      report({ status: "failed", reason: "rate_limited" });
      expect(within(replies()).getByRole("alert")).toHaveTextContent(
        /Suggestions could not be prepared: the provider is limiting requests/,
      );
      expect(within(replies()).getAllByRole("article")).toHaveLength(2);
    });

    it("labels diacritics in an Arabic reply as AI-generated, and lets the user edit them", async () => {
      await captureWithReplies();
      report({
        status: "ready",
        suggestions: [
          { style: "concise", text: "مَرْحَبًا", meaning: "Salam", asksForMissingFacts: false },
        ],
      });
      expect(
        within(replies()).getByText(/diacritics in this text were added by AI/),
      ).toBeInTheDocument();

      fireEvent.click(within(replies()).getByRole("button", { name: "Edit" }));
      fireEvent.change(within(replies()).getByRole("textbox"), { target: { value: "مرحبا" } });
      fireEvent.click(within(replies()).getByRole("button", { name: "Done" }));
      // With the diacritics removed by hand, the note goes away.
      expect(within(replies()).queryByText(/added by AI/)).not.toBeInTheDocument();
    });

    it("has no Speak button unless speech is turned on", async () => {
      await captureWithReplies();
      report({ status: "ready", suggestions });
      expect(within(replies()).queryByRole("button", { name: /Speak/ })).not.toBeInTheDocument();
    });

    it("offers no way to send a reply", async () => {
      await captureWithReplies();
      report({ status: "ready", suggestions });
      const buttons = within(replies())
        .getAllByRole("button")
        .map((button) => button.textContent);
      expect(buttons).not.toContainEqual(expect.stringMatching(/send/i));
    });
  });

  describe("demo data", () => {
    it("shows empty panels and no demo content by default", () => {
      renderApp();
      expect(screen.getByText(/No transcript yet/)).toBeInTheDocument();
      expect(screen.getByText(/No suggestions yet/)).toBeInTheDocument();
      expect(screen.queryByText("Demo")).not.toBeInTheDocument();
      expect(screen.queryByText(/Demo mode/)).not.toBeInTheDocument();
    });

    it("labels every panel as demo when demo data is shown", () => {
      renderApp();
      showDemoData();
      expect(
        screen.getByText(/Demo mode\. The text below is fixed sample data/),
      ).toBeInTheDocument();
      for (const title of ["Conversation", "Reply suggestions"]) {
        const panel = screen.getByRole("region", { name: new RegExp(`^${title}`) });
        expect(within(panel).getByText("Demo")).toBeInTheDocument();
      }
      expect(screen.getByText("Idle")).toBeInTheDocument();
    });

    it("hides demo data while capturing, so it cannot pass for a live session", async () => {
      renderApp();
      showDemoData();
      await startCapture();
      expect(screen.queryByText(/Demo mode/)).not.toBeInTheDocument();
      expect(screen.getByText(/No transcript yet/)).toBeInTheDocument();
      expect(screen.getByRole("checkbox", { name: "Show demo data" })).toBeDisabled();
    });

    it("marks partial transcript segments as in progress", () => {
      renderApp();
      showDemoData();
      expect(within(conversation()).getAllByRole("listitem")).toHaveLength(
        DEMO_CONVERSATION.segments.length,
      );
      expect(within(conversation()).getAllByText("In progress")).toHaveLength(1);
    });

    it("shows three reply suggestions, one with a missing-facts warning", () => {
      renderApp();
      showDemoData();
      const replies = screen.getByRole("region", { name: /^Reply suggestions/ });
      expect(within(replies).getAllByRole("article")).toHaveLength(3);
      for (const style of ["Professional", "Friendly", "Concise"]) {
        expect(within(replies).getByRole("heading", { name: style })).toBeInTheDocument();
      }
      expect(within(replies).getAllByText(/asks a question because a fact/)).toHaveLength(1);
    });

    it("copies the edited reply text", async () => {
      renderApp();
      showDemoData();
      const card = screen.getAllByRole("article")[2];
      if (card === undefined) {
        throw new Error("Expected a third reply card.");
      }

      fireEvent.click(within(card).getByRole("button", { name: "Edit" }));
      fireEvent.change(within(card).getByRole("textbox", { name: "Edit the Concise reply" }), {
        target: { value: "Understood, thank you." },
      });
      fireEvent.click(within(card).getByRole("button", { name: "Done" }));
      fireEvent.click(within(card).getByRole("button", { name: "Copy" }));

      expect(writeText).toHaveBeenCalledWith("Understood, thank you.");
      expect(await within(card).findByText("Copied")).toBeInTheDocument();
    });

    it("reports a failed copy", async () => {
      writeText.mockRejectedValue(new Error("denied"));
      renderApp();
      showDemoData();
      const card = screen.getAllByRole("article")[0];
      if (card === undefined) {
        throw new Error("Expected a reply card.");
      }
      fireEvent.click(within(card).getByRole("button", { name: "Copy" }));
      expect(await within(card).findByText("Could not copy")).toBeInTheDocument();
    });
  });
});
