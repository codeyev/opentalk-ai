import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeCapture, fakeCaptureClientModule } from "../../test/fakeCapture";
import { fakeProvider, fakeProviderClientModule } from "../../test/fakeProvider";
import { renderApp } from "../../test/renderApp";

vi.mock("../../capture/captureClient", () => fakeCaptureClientModule());
vi.mock("../../providers/providerClient", () => fakeProviderClientModule());

async function openProviders() {
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Providers" }));
  await screen.findAllByText("OpenAI");
}

function translationCheckbox(provider = /OpenAI/) {
  return screen.getByRole("checkbox", {
    name: new RegExp(`Send transcript text to ${provider.source}`),
  });
}

/** The transcription panel, which holds OpenAI's API key field. */
function transcriptionPanel() {
  return within(screen.getByRole("region", { name: "Transcription" }));
}

function enableCheckbox() {
  return screen.getByRole("checkbox", { name: /Send captured speech to OpenAI/ });
}

describe("Providers screen", () => {
  beforeEach(() => {
    fakeCapture.reset();
    fakeProvider.reset();
  });

  it("discloses that speech is sent to a cloud service", async () => {
    await openProviders();
    expect(
      transcriptionPanel().getByRole("radio", { name: "OpenAI · cloud service" }),
    ).toBeChecked();
    expect(screen.getByText(/parts of the captured audio that contain speech/)).toBeInTheDocument();
    expect(screen.getByText(/Does not publish a list of supported languages/)).toBeInTheDocument();
  });

  it("keeps transcription off and locked until a key is saved", async () => {
    await openProviders();
    expect(transcriptionPanel().getByText("No API key is saved.")).toBeInTheDocument();
    expect(enableCheckbox()).toBeDisabled();
    expect(enableCheckbox()).not.toBeChecked();
  });

  it("saves a key, clears the field and unlocks transcription", async () => {
    await openProviders();
    const field = transcriptionPanel().getByLabelText("API key");
    fireEvent.change(field, { target: { value: "secret-key" } });
    fireEvent.click(transcriptionPanel().getByRole("button", { name: "Save key" }));

    expect(await transcriptionPanel().findByText(/An API key is saved/)).toBeInTheDocument();
    expect(fakeProvider.saveApiKey).toHaveBeenCalledWith({ owner: "openai" }, "secret-key");
    expect(field).toHaveValue("");
    expect(enableCheckbox()).toBeEnabled();
    // Saving a key does not turn transcription on by itself.
    expect(enableCheckbox()).not.toBeChecked();
  });

  it("never stores the key in the app's settings", async () => {
    await openProviders();
    fireEvent.change(transcriptionPanel().getByLabelText("API key"), {
      target: { value: "secret-key" },
    });
    fireEvent.click(transcriptionPanel().getByRole("button", { name: "Save key" }));
    await transcriptionPanel().findByText(/An API key is saved/);
    expect(JSON.stringify(window.localStorage)).not.toContain("secret-key");
  });

  it("explains a rejected key", async () => {
    fakeProvider.saveApiKey.mockRejectedValue({ code: "invalid_key" });
    await openProviders();
    fireEvent.change(transcriptionPanel().getByLabelText("API key"), {
      target: { value: "bad key" },
    });
    fireEvent.click(transcriptionPanel().getByRole("button", { name: "Save key" }));
    expect(await transcriptionPanel().findByRole("alert")).toHaveTextContent(
      /That key is not valid/,
    );
  });

  it("turns transcription off again when the key is removed", async () => {
    fakeProvider.reset({ hasApiKey: true });
    await openProviders();
    fireEvent.click(enableCheckbox());
    expect(enableCheckbox()).toBeChecked();

    fireEvent.click(transcriptionPanel().getByRole("button", { name: "Remove key" }));
    await transcriptionPanel().findByText("No API key is saved.");
    expect(fakeProvider.deleteApiKey).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(enableCheckbox()).not.toBeChecked();
    });
    expect(enableCheckbox()).toBeDisabled();
  });

  it("says so when the core cannot be reached", async () => {
    fakeProvider.status = null;
    renderApp();
    fireEvent.click(screen.getByRole("button", { name: "Providers" }));
    expect(
      await screen.findByText("Transcription settings are available in the desktop app."),
    ).toBeInTheDocument();
  });

  describe("reply suggestions", () => {
    function repliesCheckbox() {
      return screen.getByRole("checkbox", { name: /Send the conversation to OpenAI/ });
    }

    it("discloses what is sent and that nothing is sent for the user", async () => {
      await openProviders();
      expect(
        screen.getByText(/recent conversation .* and your business context/),
      ).toBeInTheDocument();
      expect(screen.getByText(/never sends anything on your behalf/)).toBeInTheDocument();
    });

    it("is off by default and needs a key and transcription", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      expect(repliesCheckbox()).toBeDisabled();

      fireEvent.click(enableCheckbox());
      expect(repliesCheckbox()).toBeEnabled();
      expect(repliesCheckbox()).not.toBeChecked();
    });
  });

  describe("translation", () => {
    it("discloses what is sent and claims no language list", async () => {
      await openProviders();
      expect(screen.getByText(/text of each finished transcript segment/)).toBeInTheDocument();
      expect(
        screen.getByText(/Does not publish a list of supported language pairs/),
      ).toBeInTheDocument();
    });

    it("cannot be turned on without a key and transcription", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      expect(translationCheckbox()).toBeDisabled();
      expect(screen.getByText(/Translation needs transcription turned on/)).toBeInTheDocument();

      fireEvent.click(enableCheckbox());
      expect(translationCheckbox()).toBeEnabled();
      expect(translationCheckbox()).not.toBeChecked();
    });

    it("is switched off again when transcription is switched off", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      fireEvent.click(enableCheckbox());
      fireEvent.click(translationCheckbox());
      expect(translationCheckbox()).toBeChecked();

      fireEvent.click(enableCheckbox());
      expect(translationCheckbox()).not.toBeChecked();
      expect(translationCheckbox()).toBeDisabled();
    });
  });

  describe("local AI", () => {
    const QWEN = { name: "qwen3.5:9b", sizeBytes: 7_600_000_000 };

    function translationProvider(name: RegExp) {
      const panel = screen.getByRole("region", { name: "Translation" });
      return within(panel).getByRole("radio", { name });
    }

    it("explains when Ollama is not running", async () => {
      await openProviders();
      expect(await screen.findByText(/No Ollama server is answering/)).toBeInTheDocument();
      expect(screen.queryByLabelText("Model")).not.toBeInTheDocument();
    });

    it("lists the installed models with their size", async () => {
      fakeProvider.ollama = { version: "0.40.14", models: [QWEN] };
      await openProviders();
      expect(await screen.findByText("Ollama 0.40.14 is running.")).toBeInTheDocument();
      expect(screen.getByRole("option", { name: "qwen3.5:9b · 7.6 GB" })).toBeInTheDocument();
    });

    it("explains that models are never downloaded when none is installed", async () => {
      fakeProvider.ollama = { version: "0.40.14", models: [] };
      await openProviders();
      expect(await screen.findByText(/OpenTalk AI never downloads models/)).toBeInTheDocument();
    });

    it("checks again when asked", async () => {
      await openProviders();
      await screen.findByText(/No Ollama server is answering/);
      fakeProvider.ollama = { version: "0.40.14", models: [QWEN] };
      fireEvent.click(screen.getByRole("button", { name: "Check connection" }));
      expect(await screen.findByText("Ollama 0.40.14 is running.")).toBeInTheDocument();
    });

    it("needs a chosen model before a local provider can be turned on", async () => {
      fakeProvider.reset({ hasApiKey: true });
      fakeProvider.ollama = { version: "0.40.14", models: [QWEN] };
      await openProviders();
      fireEvent.click(enableCheckbox());
      fireEvent.click(translationProvider(/Ollama · on this computer/));
      expect(screen.getByText("Choose a local model under Local AI first.")).toBeInTheDocument();
      expect(translationCheckbox(/Ollama/)).toBeDisabled();

      fireEvent.change(await screen.findByLabelText("Model"), { target: { value: QWEN.name } });
      expect(screen.getByText(/Handled by the model qwen3.5:9b/)).toBeInTheDocument();
      expect(translationCheckbox(/Ollama/)).toBeEnabled();
    });
  });

  describe("other AI service", () => {
    function customPanel() {
      return within(screen.getByRole("region", { name: "Other AI service" }));
    }

    function translationPanel() {
      return within(screen.getByRole("region", { name: "Translation" }));
    }

    it("starts with OpenRouter's address as an example and no key", async () => {
      await openProviders();
      expect(customPanel().getByLabelText("Service address")).toHaveValue(
        "https://openrouter.ai/api/v1",
      );
      expect(customPanel().getByText("No API key is saved.")).toBeInTheDocument();
      expect(customPanel().getByText(/handled under its own policies/)).toBeInTheDocument();
    });

    it("saves its key separately from OpenAI's", async () => {
      await openProviders();
      fireEvent.change(customPanel().getByLabelText("API key"), { target: { value: "sk-or-key" } });
      fireEvent.click(customPanel().getByRole("button", { name: "Save key" }));

      expect(await customPanel().findByText(/An API key is saved/)).toBeInTheDocument();
      expect(fakeProvider.saveApiKey).toHaveBeenCalledWith(
        { owner: "custom", baseUrl: "https://openrouter.ai/api/v1" },
        "sk-or-key",
      );
      // OpenAI still has no key.
      expect(transcriptionPanel().getByText("No API key is saved.")).toBeInTheDocument();
      expect(JSON.stringify(window.localStorage)).not.toContain("sk-or-key");
    });

    it("refuses an address that is not https", async () => {
      await openProviders();
      const address = customPanel().getByLabelText("Service address");
      fireEvent.change(address, { target: { value: "http://openrouter.ai/api/v1" } });
      expect(address).toHaveAttribute("aria-invalid", "true");
      expect(customPanel().getByText(/Enter an https address/)).toBeInTheDocument();

      fireEvent.change(address, { target: { value: "http://localhost:1234/v1" } });
      expect(address).toHaveAttribute("aria-invalid", "false");
    });

    it("can be chosen for translation once its address, model and key are set", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      fireEvent.click(enableCheckbox());
      fireEvent.click(
        translationPanel().getByRole("radio", { name: "openrouter.ai · cloud service" }),
      );
      expect(
        translationPanel().getByText(/Enter the service address, a model name/),
      ).toBeInTheDocument();
      expect(translationCheckbox(/openrouter\.ai/)).toBeDisabled();

      fireEvent.change(customPanel().getByLabelText("Model name"), {
        target: { value: "openai/gpt-4o-mini" },
      });
      fireEvent.change(customPanel().getByLabelText("API key"), { target: { value: "sk-or-key" } });
      fireEvent.click(customPanel().getByRole("button", { name: "Save key" }));
      await customPanel().findByText(/An API key is saved/);

      expect(translationCheckbox(/openrouter\.ai/)).toBeEnabled();
      expect(
        translationPanel().getByText(/sent over an encrypted connection to openrouter.ai/),
      ).toBeInTheDocument();
    });

    it("counts a service on this computer as local, with no key needed", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      fireEvent.click(enableCheckbox());
      fireEvent.change(customPanel().getByLabelText("Service address"), {
        target: { value: "http://localhost:1234/v1" },
      });
      fireEvent.change(customPanel().getByLabelText("Model name"), {
        target: { value: "local-model" },
      });

      fireEvent.click(
        translationPanel().getByRole("radio", { name: "localhost · on this computer" }),
      );
      expect(translationCheckbox(/localhost/)).toBeEnabled();
    });
  });

  describe("local-only mode", () => {
    function localOnly() {
      return screen.getByRole("checkbox", { name: "Never contact a cloud service" });
    }

    it("is off by default", async () => {
      await openProviders();
      expect(localOnly()).not.toBeChecked();
    });

    it("turns off cloud transcription and rules out cloud providers", async () => {
      fakeProvider.reset({ hasApiKey: true });
      await openProviders();
      fireEvent.click(enableCheckbox());
      expect(enableCheckbox()).toBeChecked();

      fireEvent.click(localOnly());
      // Transcription falls back to Whisper, which has no model yet, so it is off.
      expect(screen.queryByRole("checkbox", { name: /Send captured speech to OpenAI/ })).toBeNull();
      const whisper = transcriptionPanel().getByRole("checkbox", {
        name: /Transcribe captured speech/,
      });
      expect(whisper).toBeDisabled();
      expect(whisper).not.toBeChecked();
      expect(screen.getByText(/everything runs on this computer/)).toBeInTheDocument();
      for (const cloud of screen.getAllByRole("radio", { name: /OpenAI · cloud service/ })) {
        expect(cloud).toBeDisabled();
      }
      for (const local of screen.getAllByRole("radio", { name: /Ollama · on this computer/ })) {
        expect(local).toBeChecked();
      }
    });
  });

  describe("local transcription", () => {
    function whisperPanel() {
      return within(screen.getByRole("region", { name: "Local transcription" }));
    }

    it("explains what to do when no model is installed", async () => {
      await openProviders();
      expect(await whisperPanel().findByText(/No model is installed yet/)).toBeInTheDocument();
      expect(whisperPanel().getByText(/only when you press Download/)).toBeInTheDocument();
      expect(whisperPanel().getByText("Models folder: /app-data/models")).toBeInTheDocument();
    });

    it("downloads a model only when asked, then selects it", async () => {
      await openProviders();
      const download = await whisperPanel().findByRole("button", {
        name: "Download base (148 MB)",
      });
      expect(fakeProvider.downloadWhisperModel).not.toHaveBeenCalled();

      fireEvent.click(download);
      expect(await whisperPanel().findByText("base is installed")).toBeInTheDocument();
      expect(fakeProvider.downloadWhisperModel).toHaveBeenCalledWith("base", false);
      expect(whisperPanel().getByLabelText("Model")).toHaveValue("ggml-base.bin");
    });

    it("does not offer downloads in local-only mode, and says why", async () => {
      await openProviders();
      fireEvent.click(screen.getByRole("checkbox", { name: "Never contact a cloud service" }));
      expect(await whisperPanel().findByRole("button", { name: /Download base/ })).toBeDisabled();
      expect(
        whisperPanel().getByText(/Downloading is turned off in local-only mode/),
      ).toBeInTheDocument();
    });

    it("explains a download that failed verification", async () => {
      fakeProvider.downloadWhisperModel.mockRejectedValue({ code: "verification_failed" });
      await openProviders();
      fireEvent.click(await whisperPanel().findByRole("button", { name: /Download tiny/ }));
      expect(await whisperPanel().findByRole("alert")).toHaveTextContent(
        /did not match the published model/,
      );
    });

    it("can transcribe without any API key once a model is chosen", async () => {
      fakeProvider.whisper = {
        ...fakeProvider.whisper,
        installed: [{ fileName: "ggml-base.bin", sizeBytes: 147_951_465 }],
      };
      await openProviders();
      fireEvent.click(
        transcriptionPanel().getByRole("radio", { name: "Whisper · on this computer" }),
      );
      expect(
        transcriptionPanel().getAllByText(/Choose a Whisper model under Local transcription/),
      ).not.toHaveLength(0);

      fireEvent.change(await whisperPanel().findByLabelText("Model"), {
        target: { value: "ggml-base.bin" },
      });
      expect(transcriptionPanel().getByText(/No audio is sent anywhere/)).toBeInTheDocument();
      const enable = transcriptionPanel().getByRole("checkbox", {
        name: /Transcribe captured speech with Whisper/,
      });
      expect(enable).toBeEnabled();
    });
  });

  describe("service presets", () => {
    function customPanel() {
      return within(screen.getByRole("region", { name: "Other AI service" }));
    }

    it("fills in the address of a chosen service and clears the model", async () => {
      await openProviders();
      fireEvent.change(customPanel().getByLabelText("Model name"), {
        target: { value: "old-model" },
      });
      fireEvent.change(customPanel().getByLabelText("Service"), {
        target: { value: "https://api.deepseek.com" },
      });
      expect(customPanel().getByLabelText("Service address")).toHaveValue(
        "https://api.deepseek.com",
      );
      expect(customPanel().getByLabelText("Model name")).toHaveValue("");
      expect(customPanel().getByLabelText("Model name")).toHaveAttribute(
        "placeholder",
        "deepseek-flash",
      );
    });

    it("offers Gemini, Claude, DeepSeek and others", async () => {
      await openProviders();
      for (const name of [
        "OpenRouter",
        "OpenAI",
        "Google Gemini",
        "Anthropic Claude",
        "DeepSeek",
        "Groq",
      ]) {
        expect(customPanel().getByRole("option", { name })).toBeInTheDocument();
      }
    });

    it("keeps a separate key for each service", async () => {
      await openProviders();
      fireEvent.change(customPanel().getByLabelText("API key"), { target: { value: "sk-or-key" } });
      fireEvent.click(customPanel().getByRole("button", { name: "Save key" }));
      await customPanel().findByText(/An API key is saved/);

      // Switching to another service must not reuse OpenRouter's key.
      fireEvent.change(customPanel().getByLabelText("Service"), {
        target: { value: "https://api.deepseek.com" },
      });
      expect(await customPanel().findByText("No API key is saved.")).toBeInTheDocument();

      fireEvent.change(customPanel().getByLabelText("Service"), {
        target: { value: "https://openrouter.ai/api/v1" },
      });
      expect(await customPanel().findByText(/An API key is saved/)).toBeInTheDocument();
    });
  });
});
