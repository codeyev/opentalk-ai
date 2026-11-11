import { useId, useState } from "react";
import { useI18n } from "../../i18n/I18nContext";
import {
  deleteApiKey,
  type KeyErrorCode,
  type KeyTarget,
  saveApiKey,
  toKeyErrorCode,
} from "../../providers/providerClient";

interface ApiKeyFieldProps {
  /** The service the key belongs to. A key is only ever sent to that service. */
  target: KeyTarget;
  hasApiKey: boolean;
  /** Called after a key was saved or removed. */
  onChange: (change: "saved" | "removed") => Promise<void> | void;
}

/**
 * Saves or removes one service's API key. The key goes straight to the
 * system credential store; it is never shown again or kept in the page.
 */
export function ApiKeyField({ target, hasApiKey, onChange }: ApiKeyFieldProps) {
  const { t } = useI18n();
  const fieldId = useId();
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<KeyErrorCode | null>(null);

  const save = async () => {
    try {
      await saveApiKey(target, draft);
      // The key must not linger in the page once it is stored.
      setDraft("");
      setError(null);
      await onChange("saved");
    } catch (failure) {
      setError(toKeyErrorCode(failure));
    }
  };

  const remove = async () => {
    try {
      await deleteApiKey(target);
      setError(null);
      await onChange("removed");
    } catch (failure) {
      setError(toKeyErrorCode(failure));
    }
  };

  return (
    <form
      className="field"
      onSubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <label htmlFor={fieldId}>{t("providers.key.label")}</label>
      <p role="status" className="note">
        {t(hasApiKey ? "providers.key.saved" : "providers.key.missing")}
      </p>
      <div className="button-row">
        <input
          id={fieldId}
          type="password"
          className="input input-key"
          autoComplete="off"
          spellCheck={false}
          value={draft}
          aria-describedby={`${fieldId}-hint`}
          onChange={(event) => {
            setDraft(event.target.value);
          }}
        />
        <button type="submit" className="button" disabled={draft === ""}>
          {t("providers.key.save")}
        </button>
        <button
          type="button"
          className="button"
          disabled={!hasApiKey}
          onClick={() => {
            void remove();
          }}
        >
          {t("providers.key.remove")}
        </button>
      </div>
      <p id={`${fieldId}-hint`} className="note">
        {t("providers.key.hint")}
      </p>
      {error !== null && (
        <p role="alert" className="error-text">
          {t(`providers.key.error.${error}`)}
        </p>
      )}
    </form>
  );
}
