import { type ComponentType, useEffect, useState } from "react";
import { CaptureProvider } from "./capture/CaptureProvider";
import { Sidebar } from "./components/Sidebar";
import { useI18n } from "./i18n/I18nContext";
import { I18nProvider } from "./i18n/I18nProvider";
import { AboutScreen } from "./screens/AboutScreen";
import { AppearanceScreen } from "./screens/AppearanceScreen";
import { BusinessContextScreen } from "./screens/BusinessContextScreen";
import { LanguagesScreen } from "./screens/LanguagesScreen";
import { LiveWorkspace } from "./screens/live/LiveWorkspace";
import { PrivacyScreen } from "./screens/PrivacyScreen";
import { ProvidersScreen } from "./screens/providers/ProvidersScreen";
import { RtlPreviewScreen } from "./screens/RtlPreviewScreen";
import type { ScreenId } from "./screens/screens";
import { useSettings } from "./settings/SettingsContext";
import { SettingsProvider } from "./settings/SettingsProvider";

const SCREEN_COMPONENTS: Record<ScreenId, ComponentType> = {
  live: LiveWorkspace,
  languages: LanguagesScreen,
  providers: ProvidersScreen,
  privacy: PrivacyScreen,
  context: BusinessContextScreen,
  appearance: AppearanceScreen,
  rtl: RtlPreviewScreen,
  about: AboutScreen,
};

/** Applies the interface language, text direction and theme to the document root. */
function useDocumentAppearance() {
  const { uiLocale, rtlPreview, theme } = useSettings().settings;
  useEffect(() => {
    const root = document.documentElement;
    root.lang = uiLocale;
    root.dir = rtlPreview ? "rtl" : "ltr";
    root.dataset.theme = theme;
  }, [uiLocale, rtlPreview, theme]);
}

function AppShell() {
  const { t } = useI18n();
  const [screen, setScreen] = useState<ScreenId>("live");
  const Screen = SCREEN_COMPONENTS[screen];
  useDocumentAppearance();

  return (
    <div className="shell">
      <a className="skip-link" href="#main">
        {t("app.skipToContent")}
      </a>
      <Sidebar current={screen} onNavigate={setScreen} />
      <main id="main" className="content" tabIndex={-1}>
        {/* The live workspace has its own, fuller page header. */}
        {screen !== "live" && (
          <header className="page-header">
            <h1>{t(`nav.${screen}`)}</h1>
          </header>
        )}
        <Screen />
      </main>
    </div>
  );
}

export function App() {
  return (
    <SettingsProvider>
      <I18nProvider>
        <CaptureProvider>
          <AppShell />
        </CaptureProvider>
      </I18nProvider>
    </SettingsProvider>
  );
}
