import { useId, useState } from "react";
import { useI18n } from "../i18n/I18nContext";
import { canonicalLanguageTag, languageDisplayName } from "../lib/language";

interface LanguageFieldProps {
  label: string;
  /** A valid BCP 47 language tag. */
  value: string;
  onChange: (tag: string) => void;
}

/**
 * Text field for a BCP 47 language tag. Languages are not limited to a fixed
 * list, because support depends on the provider. `onChange` is only called
 * with valid tags; an invalid draft stays in the field with an error message.
 */
export function LanguageField({ label, value, onChange }: LanguageFieldProps) {
  const { t, locale } = useI18n();
  const inputId = useId();
  const noteId = useId();
  const [draft, setDraft] = useState(value);
  const draftTag = canonicalLanguageTag(draft);

  const handleChange = (text: string) => {
    setDraft(text);
    const tag = canonicalLanguageTag(text);
    if (tag !== null) {
      onChange(tag);
    }
  };

  return (
    <div className="field">
      <label htmlFor={inputId}>{label}</label>
      <input
        id={inputId}
        type="text"
        className="input input-short"
        value={draft}
        spellCheck={false}
        autoCapitalize="none"
        aria-invalid={draftTag === null}
        aria-describedby={noteId}
        onChange={(event) => {
          handleChange(event.target.value);
        }}
      />
      <p id={noteId} className={draftTag === null ? "note note-error" : "note"}>
        {draftTag === null
          ? t("languages.tagInvalid")
          : `${languageDisplayName(draftTag, locale)} · ${t("languages.tagHint")}`}
      </p>
    </div>
  );
}
