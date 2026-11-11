import { render } from "@testing-library/react";
import { App } from "../App";

/** Renders the whole app with default settings, as on first launch. */
export function renderApp() {
  window.localStorage.clear();
  return render(<App />);
}

/** Renders the whole app with whatever settings are already stored, as on a restart. */
export function renderAppWithStoredSettings() {
  return render(<App />);
}
