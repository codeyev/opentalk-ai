import { useEffect, useState } from "react";
import { useI18n } from "../i18n/I18nContext";
import { type AppInfo, fetchAppInfo } from "../lib/appInfo";

type CoreStatus =
  { state: "loading" } | { state: "ready"; info: AppInfo } | { state: "unavailable" };

export function AboutScreen() {
  const { t } = useI18n();
  const [core, setCore] = useState<CoreStatus>({ state: "loading" });

  useEffect(() => {
    let isCurrent = true;
    fetchAppInfo().then(
      (info) => {
        if (isCurrent) {
          setCore({ state: "ready", info });
        }
      },
      () => {
        if (isCurrent) {
          setCore({ state: "unavailable" });
        }
      },
    );
    return () => {
      isCurrent = false;
    };
  }, []);

  return (
    <div className="form">
      <p className="intro">{t("about.description")}</p>
      <p className="notice">{t("about.status")}</p>
      <p role="status">
        {core.state === "loading" && t("about.core.loading")}
        {core.state === "ready" && t("about.core.ready", { ...core.info })}
        {core.state === "unavailable" && t("about.core.unavailable")}
      </p>
      <p>{t("about.license")}</p>
    </div>
  );
}
