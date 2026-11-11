import { useI18n } from "../i18n/I18nContext";

/** Small text label marking a panel whose content is demo data. */
export function DemoTag() {
  const { t } = useI18n();
  return <span className="demo-tag">{t("common.demoTag")}</span>;
}
