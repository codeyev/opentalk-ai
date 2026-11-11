import { CopyButton } from "../components/CopyButton";
import { useI18n } from "../i18n/I18nContext";
import { useSettings } from "../settings/SettingsContext";

/**
 * Fixed Arabic samples for checking right-to-left layout. Each string is
 * stored in logical order, which is also the order the Copy button copies.
 */
const SAMPLES = [
  { id: "plain", text: "مرحبًا، كيف يمكنني مساعدتك اليوم؟" },
  { id: "tashkeel", text: "مَرْحَبًا، كَيْفَ يُمْكِنُنِي مُسَاعَدَتُكَ الْيَوْمَ؟" },
  { id: "mixed", text: "سيتم تسليم المشروع عبر GitHub خلال 14 يومًا بسعر 1,200 USD." },
  { id: "url", text: "يمكنك زيارة الموقع https://example.com/docs لمعرفة المزيد." },
] as const;

export function RtlPreviewScreen() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();

  return (
    <div className="form">
      <p className="intro">{t("rtl.intro")}</p>
      <label className="choice">
        <input
          type="checkbox"
          checked={settings.rtlPreview}
          onChange={(event) => {
            updateSettings({ rtlPreview: event.target.checked });
          }}
        />
        {t("rtl.toggle")}
      </label>

      <section className="panel" aria-labelledby="rtl-samples-title">
        <h2 id="rtl-samples-title">{t("rtl.samples")}</h2>
        <ul className="rtl-samples">
          {SAMPLES.map((sample) => (
            <li key={sample.id} className="rtl-sample">
              <h3>{t(`rtl.sample.${sample.id}`)}</h3>
              <p className="arabic" lang="ar" dir="rtl">
                {sample.text}
              </p>
              <CopyButton text={sample.text} />
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
