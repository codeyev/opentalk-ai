import { type ReactNode, useCallback, useEffect, useMemo, useState } from "react";
import { DEFAULT_SETTINGS, type Settings } from "./settings";
import { SettingsContext, type SettingsContextValue } from "./SettingsContext";
import { loadSettings, saveSettings } from "./settingsStorage";

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(loadSettings);

  useEffect(() => {
    saveSettings(settings);
  }, [settings]);

  const updateSettings = useCallback((changes: Partial<Settings>) => {
    setSettings((current) => ({ ...current, ...changes }));
  }, []);

  const resetSettings = useCallback(() => {
    setSettings(DEFAULT_SETTINGS);
  }, []);

  const value = useMemo<SettingsContextValue>(
    () => ({ settings, updateSettings, resetSettings }),
    [settings, updateSettings, resetSettings],
  );

  return <SettingsContext value={value}>{children}</SettingsContext>;
}
