import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fakeCapture, fakeCaptureClientModule } from "../test/fakeCapture";
import { fakeProvider, fakeProviderClientModule } from "../test/fakeProvider";
import { installFakeSpeech } from "../test/fakeSpeech";
import { renderApp } from "../test/renderApp";

vi.mock("../capture/captureClient", () => fakeCaptureClientModule());
vi.mock("../providers/providerClient", () => fakeProviderClientModule());

function open(screenName: string) {
  fireEvent.click(screen.getByRole("button", { name: screenName }));
}

/** Rejects with a plain value, the way a failed command from the core does. */
function rejectWith(value: unknown): Promise<never> {
  // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
  return Promise.reject(value);
}

function speechPanel() {
  return within(screen.getByRole("region", { name: "Speech" }));
}

function readAloudSwitch() {
  return speechPanel().getByRole("checkbox", { name: "Read translations of the other side aloud" });
}

/** Turns on transcription and translation, lets `configure` set up speech, and starts capture. */
async function startSession(configure: () => void) {
  renderApp();
  open("Providers");
  fireEvent.click(await screen.findByRole("checkbox", { name: /Send captured speech to OpenAI/ }));
  fireEvent.click(screen.getByRole("checkbox", { name: /Send transcript text to OpenAI/ }));
  configure();
  open("Live workspace");
  await screen.findByText(/Translation into English is on/);
  fireEvent.click(screen.getByRole("button", { name: "Start" }));
  await screen.findByText("Capturing audio");
}

/** Reports one finished thing said, and its translation. */
function say(source: "system" | "microphone", id: string, translation: string) {
  act(() => {
    fakeCapture.events().onTranscript({
      kind: "segment",
      segment: { id, source, startMs: 0, text: "…", language: "ru", isFinal: true },
    });
    fakeCapture.events().onTranslation({
      segmentId: id,
      state: { status: "done", text: translation, targetLanguage: "en", unpreserved: [] },
    });
  });
}

/** A stand-in for the webview's audio output, which plays each buffer at once. */
function installFakeAudioOutput() {
  class FakeAudioContext {
    destination = {};
    resume = () => Promise.resolve();
    createBuffer = () => ({ copyToChannel: () => undefined });
    createBufferSource = () => {
      let onEnded: () => void = () => undefined;
      return {
        buffer: null,
        connect: () => undefined,
        addEventListener: (_type: string, listener: () => void) => {
          onEnded = listener;
        },
        start: () => {
          onEnded();
        },
        stop: () => undefined,
      };
    };
  }
  vi.stubGlobal("AudioContext", FakeAudioContext);
}

describe("Reading translations aloud", () => {
  beforeEach(() => {
    fakeCapture.reset();
    fakeProvider.reset({ hasApiKey: true });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("is off by default, so nothing is spoken on its own", async () => {
    const speech = installFakeSpeech(["en-US"]);
    await startSession(() => {
      expect(readAloudSwitch()).not.toBeChecked();
    });

    say("system", "system-1", "Good morning.");
    expect(speech.speak).not.toHaveBeenCalled();
    expect(screen.queryByText(/read aloud/)).not.toBeInTheDocument();
  });

  it("speaks each translation of the other side once, and never the user's own words", async () => {
    const speech = installFakeSpeech(["en-US"]);
    await startSession(() => {
      fireEvent.click(readAloudSwitch());
    });
    expect(screen.getByText(/read aloud by System voices on this computer/)).toBeInTheDocument();

    say("system", "system-1", "Good morning.");
    say("microphone", "microphone-1", "Hello.");
    say("system", "system-2", "How are you?");
    // A translation that is reported again is not repeated.
    say("system", "system-1", "Good morning.");

    expect(speech.speak.mock.calls.map(([utterance]) => utterance.text)).toEqual([
      "Good morning.",
      "How are you?",
    ]);
  });

  it("falls silent when capture stops", async () => {
    const speech = installFakeSpeech(["en-US"]);
    await startSession(() => {
      fireEvent.click(readAloudSwitch());
    });
    say("system", "system-1", "Good morning.");
    expect(speech.cancel).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    await waitFor(() => {
      expect(speech.cancel).toHaveBeenCalled();
    });
  });

  it("warns when no system voice speaks the target language", () => {
    installFakeSpeech(["ar-SA"]);
    renderApp();
    open("Providers");
    fireEvent.click(readAloudSwitch());

    expect(speechPanel().getByText(/No system voice is installed for English/)).toBeInTheDocument();
    expect(speechPanel().getByText(/Translation is off/)).toBeInTheDocument();
  });

  describe("with ElevenLabs", () => {
    function chooseElevenLabs() {
      fireEvent.click(speechPanel().getByRole("radio", { name: "ElevenLabs, a cloud service" }));
      fireEvent.change(speechPanel().getByLabelText("Voice ID"), { target: { value: "voice-1" } });
    }

    it("says where the text goes and keeps the key in the credential store", async () => {
      installFakeAudioOutput();
      renderApp();
      open("Providers");
      chooseElevenLabs();

      expect(
        speechPanel().getByText(/sent over an encrypted connection to ElevenLabs/),
      ).toBeInTheDocument();
      expect(await speechPanel().findByText("No API key is saved.")).toBeInTheDocument();
      fireEvent.change(speechPanel().getByLabelText("API key"), { target: { value: "secret" } });
      fireEvent.click(speechPanel().getByRole("button", { name: "Save key" }));

      expect(await speechPanel().findByText(/An API key is saved/)).toBeInTheDocument();
      expect(fakeProvider.saveApiKey).toHaveBeenCalledWith({ owner: "elevenlabs" }, "secret");
      expect(speechPanel().getByLabelText("API key")).toHaveValue("");
    });

    it("sends each translation to the chosen voice", async () => {
      installFakeAudioOutput();
      await startSession(() => {
        chooseElevenLabs();
        fireEvent.click(readAloudSwitch());
      });
      expect(
        screen.getByText(/their text is sent to ElevenLabs, a cloud service/),
      ).toBeInTheDocument();

      say("system", "system-1", "Good morning.");
      await waitFor(() => {
        expect(fakeProvider.synthesizeSpeech).toHaveBeenCalledExactlyOnceWith(
          { id: "elevenlabs", voice: "voice-1", model: "eleven_v3" },
          "Good morning.",
          "en",
          false,
        );
      });
    });

    it("says why a translation could not be read aloud", async () => {
      installFakeAudioOutput();
      fakeProvider.synthesizeSpeech.mockImplementation(() =>
        rejectWith({ code: "missing_api_key" }),
      );
      await startSession(() => {
        chooseElevenLabs();
        fireEvent.click(readAloudSwitch());
      });

      say("system", "system-1", "Good morning.");
      expect(
        await screen.findByText("A translation could not be read aloud: no API key is saved."),
      ).toBeInTheDocument();
    });

    it("is ruled out by local-only mode", () => {
      installFakeAudioOutput();
      renderApp();
      open("Providers");
      chooseElevenLabs();
      fireEvent.click(readAloudSwitch());
      fireEvent.click(screen.getByRole("checkbox", { name: "Never contact a cloud service" }));

      expect(readAloudSwitch()).toBeDisabled();
      expect(readAloudSwitch()).not.toBeChecked();
      expect(speechPanel().getByText("Turned off by local-only mode.")).toBeInTheDocument();
      expect(
        speechPanel().getByRole("radio", { name: "ElevenLabs, a cloud service" }),
      ).toBeDisabled();
    });
  });
});
