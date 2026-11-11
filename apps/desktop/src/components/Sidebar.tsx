import { useI18n } from "../i18n/I18nContext";
import { type ScreenId, SETTINGS_SCREENS } from "../screens/screens";
import { Icon, type IconName } from "./Icon";
import { SessionCard } from "./SessionCard";

const SCREEN_ICONS: Record<ScreenId, IconName> = {
  live: "activity",
  languages: "languages",
  providers: "layers",
  privacy: "shield",
  context: "briefcase",
  appearance: "sliders",
  rtl: "textDirection",
  about: "info",
};

interface SidebarProps {
  current: ScreenId;
  onNavigate: (screen: ScreenId) => void;
}

export function Sidebar({ current, onNavigate }: SidebarProps) {
  const { t } = useI18n();

  const navButton = (screen: ScreenId) => (
    <li key={screen}>
      <button
        type="button"
        className="nav-button"
        aria-current={screen === current ? "page" : undefined}
        onClick={() => {
          onNavigate(screen);
        }}
      >
        <Icon name={SCREEN_ICONS[screen]} />
        {t(`nav.${screen}`)}
      </button>
    </li>
  );

  return (
    <div className="sidebar">
      <p className="brand">{t("app.name")}</p>
      <nav aria-label={t("nav.label")}>
        <ul className="nav-list">{navButton("live")}</ul>
        <p className="nav-group" id="nav-settings-group">
          {t("nav.settingsGroup")}
        </p>
        <ul className="nav-list" aria-labelledby="nav-settings-group">
          {SETTINGS_SCREENS.map(navButton)}
        </ul>
      </nav>
      <SessionCard />
    </div>
  );
}
