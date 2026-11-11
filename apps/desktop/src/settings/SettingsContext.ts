import { createContext, useContext } from "react";
import type { Settings } from "./settings";

export interface SettingsContextValue {
  settings: Settings;
  /** Merges `changes` into the current settings and saves them. */
  updateSettings: (changes: Partial<Settings>) => void;
  resetSettings: () => void;
}

export const SettingsContext = createContext<SettingsContextValue | null>(null);

export function useSettings(): SettingsContextValue {
  const value = useContext(SettingsContext);
  if (value === null) {
    throw new Error("useSettings must be used inside a SettingsProvider.");
  }
  return value;
}
