import { fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderApp, renderAppWithStoredSettings } from "./test/renderApp";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => undefined)) }));

function openScreen(name: string) {
  fireEvent.click(screen.getByRole("button", { name }));
}

describe("App shell", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("starts on the live workspace and marks it as the current page", () => {
    renderApp();
    expect(screen.getByRole("heading", { level: 1, name: "Live workspace" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Live workspace" })).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  it("navigates to every settings screen", () => {
    renderApp();
    const screens = [
      "Languages",
      "Providers",
      "Privacy",
      "Business context",
      "Appearance",
      "RTL preview",
      "About",
    ];
    for (const name of screens) {
      openScreen(name);
      expect(screen.getByRole("heading", { level: 1, name })).toBeInTheDocument();
    }
  });

  it("switches the interface language and sets the document language", () => {
    renderApp();
    openScreen("Appearance");
    fireEvent.click(screen.getByRole("radio", { name: "Azərbaycanca" }));
    expect(screen.getByRole("heading", { level: 1, name: "Görünüş" })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("az");

    fireEvent.click(screen.getByRole("radio", { name: "Русский" }));
    expect(screen.getByRole("heading", { level: 1, name: "Оформление" })).toBeInTheDocument();
  });

  it("applies the chosen theme to the document", () => {
    renderApp();
    expect(document.documentElement.dataset.theme).toBe("system");
    openScreen("Appearance");
    fireEvent.click(screen.getByRole("radio", { name: "Dark" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("lays the interface out right-to-left in RTL preview", () => {
    renderApp();
    expect(document.documentElement.dir).toBe("ltr");
    openScreen("RTL preview");
    fireEvent.click(screen.getByRole("checkbox", { name: /right-to-left/ }));
    expect(document.documentElement.dir).toBe("rtl");
  });

  it("keeps settings after the app is reopened", () => {
    const first = renderApp();
    openScreen("Appearance");
    fireEvent.click(screen.getByRole("radio", { name: "Dark" }));
    first.unmount();

    renderAppWithStoredSettings();
    openScreen("Appearance");
    expect(screen.getByRole("radio", { name: "Dark" })).toBeChecked();
  });

  it("does not allow session history to be turned on", () => {
    renderApp();
    openScreen("Privacy");
    expect(screen.getByRole("checkbox", { name: /Save session history/ })).toBeDisabled();
  });

  it("shows the core version on the About screen", async () => {
    invoke.mockResolvedValue({ name: "opentalk-desktop", version: "0.1.0" });
    renderApp();
    openScreen("About");
    expect(await screen.findByText(/opentalk-desktop 0\.1\.0/)).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("app_info");
  });

  it("reports the core as unavailable when its reply is malformed", async () => {
    invoke.mockResolvedValue({ name: 42 });
    renderApp();
    openScreen("About");
    expect(await screen.findByText(/Application core unavailable/)).toBeInTheDocument();
  });

  it("copies mixed Arabic and Latin sample text in logical order", () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    renderApp();
    openScreen("RTL preview");

    const sample = screen.getByText(/GitHub/);
    expect(sample).toHaveAttribute("dir", "rtl");
    expect(sample).toHaveAttribute("lang", "ar");
    const copy = sample.parentElement?.querySelector("button");
    if (!copy) {
      throw new Error("Expected a copy button next to the sample.");
    }
    fireEvent.click(copy);
    expect(writeText).toHaveBeenCalledWith(
      "سيتم تسليم المشروع عبر GitHub خلال 14 يومًا بسعر 1,200 USD.",
    );
  });

  it("adds and removes glossary terms", () => {
    renderApp();
    openScreen("Business context");
    expect(screen.getByText("No terms yet.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Add term" }));
    const entry = screen.getByRole("group", { name: "Term 1" });
    fireEvent.change(within(entry).getByLabelText("Term"), { target: { value: "invoice" } });
    expect(within(entry).getByLabelText("Term")).toHaveValue("invoice");

    fireEvent.click(screen.getByRole("button", { name: "Remove term 1" }));
    expect(screen.getByText("No terms yet.")).toBeInTheDocument();
  });

  it("rejects an invalid language tag and keeps the last valid one", () => {
    renderApp();
    openScreen("Languages");
    const field = screen.getByLabelText("Translate into");
    fireEvent.change(field, { target: { value: "not a language" } });
    expect(field).toHaveAttribute("aria-invalid", "true");

    fireEvent.change(field, { target: { value: "AR-eg" } });
    expect(field).toHaveAttribute("aria-invalid", "false");
    expect(screen.getByText(/Arabic \(Egypt\)/)).toBeInTheDocument();
  });
});
