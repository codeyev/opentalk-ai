import { useEffect, useState } from "react";
import { useI18n } from "../i18n/I18nContext";
import { Icon } from "./Icon";

type CopyStatus = "idle" | "copied" | "failed";

const STATUS_RESET_DELAY_MS = 2000;

/** Copies `text` to the clipboard and briefly reports whether it worked. */
export function CopyButton({ text }: { text: string }) {
  const { t } = useI18n();
  const [status, setStatus] = useState<CopyStatus>("idle");

  useEffect(() => {
    if (status === "idle") {
      return;
    }
    const timer = window.setTimeout(() => {
      setStatus("idle");
    }, STATUS_RESET_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [status]);

  const copy = () => {
    navigator.clipboard.writeText(text).then(
      () => {
        setStatus("copied");
      },
      () => {
        setStatus("failed");
      },
    );
  };

  return (
    <span className="copy">
      <button type="button" className="button button-quiet" onClick={copy}>
        <Icon name="copy" />
        {t("common.copy")}
      </button>
      <span role="status" className="copy-status">
        {status === "copied" && t("common.copied")}
        {status === "failed" && t("common.copyFailed")}
      </span>
    </span>
  );
}
