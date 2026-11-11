import { DEFAULT_SETTINGS, parseSettings, type Settings } from "./settings";

const STORAGE_KEY = "opentalk.settings.v1";

/** Reads saved settings. Falls back to defaults if storage is empty, corrupt or blocked. */
export function loadSettings(): Settings {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return stored === null ? DEFAULT_SETTINGS : parseSettings(JSON.parse(stored));
  } catch {
    return DEFAULT_SETTINGS;
  }
}

/** Saves settings. A storage failure is ignored: the settings still apply for this run. */
export function saveSettings(settings: Settings): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Storage can be unavailable or full; there is nothing useful to do about it here.
  }
}
