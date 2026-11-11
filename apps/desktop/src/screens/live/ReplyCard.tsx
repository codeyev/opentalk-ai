import { useState } from "react";
import { CopyButton } from "../../components/CopyButton";
import { DiacriticsNote } from "../../components/DiacriticsNote";
import { Icon } from "../../components/Icon";
import type { ReplySuggestion } from "../../domain/conversation";
import { useI18n } from "../../i18n/I18nContext";
import { useSpeech } from "../../speech/useSpeech";

interface ReplyCardProps {
  reply: ReplySuggestion;
  /** The card's position in the list, starting at 1. */
  number: number;
  replyLanguage: string;
  meaningLanguage: string | null;
  onDismiss: () => void;
}

/**
 * One suggested reply. The user can edit the text, copy it, or dismiss it;
 * the app never sends a reply anywhere itself.
 */
export function ReplyCard({
  reply,
  number,
  replyLanguage,
  meaningLanguage,
  onDismiss,
}: ReplyCardProps) {
  const { t } = useI18n();
  const { synthesizer } = useSpeech();
  const [text, setText] = useState(reply.text);
  const [isEditing, setIsEditing] = useState(false);
  const styleLabel = t(`live.replies.style.${reply.style}`);

  return (
    <article className="reply">
      <h3 className="reply-style">
        <span className="reply-number" aria-hidden="true">
          {number}
        </span>
        {styleLabel}
      </h3>
      {isEditing ? (
        <textarea
          className="input reply-editor"
          aria-label={t("live.replies.editLabel", { style: styleLabel })}
          lang={replyLanguage}
          dir="auto"
          rows={4}
          value={text}
          onChange={(event) => {
            setText(event.target.value);
          }}
        />
      ) : (
        <p className="reply-text" lang={replyLanguage} dir="auto">
          {text}
        </p>
      )}
      <DiacriticsNote text={text} />
      {reply.meaning !== null && (
        <p className="reply-meaning">
          <span className="visually-hidden">{t("live.replies.meaning")}: </span>
          <span lang={meaningLanguage ?? undefined} dir="auto">
            {reply.meaning}
          </span>
        </p>
      )}
      {reply.asksForMissingFacts && (
        <p className="warning">{t("live.replies.asksForMissingFacts")}</p>
      )}
      <div className="reply-actions">
        <CopyButton text={text} />
        <button
          type="button"
          className="button button-quiet"
          onClick={() => {
            setIsEditing(!isEditing);
          }}
        >
          <Icon name="pencil" />
          {isEditing ? t("live.replies.doneEditing") : t("live.replies.edit")}
        </button>
        {synthesizer !== null && (
          <button
            type="button"
            className="button button-quiet"
            aria-label={t("live.replies.speakLabel", { style: styleLabel })}
            disabled={!synthesizer.canSpeak(replyLanguage)}
            title={synthesizer.canSpeak(replyLanguage) ? undefined : t("live.replies.noVoice")}
            onClick={() => {
              synthesizer.speak(text, replyLanguage);
            }}
          >
            <Icon name="volume" />
            {t("live.replies.speak")}
          </button>
        )}
        <button
          type="button"
          className="button button-quiet"
          aria-label={t("live.replies.dismissLabel", { style: styleLabel })}
          onClick={onDismiss}
        >
          <Icon name="x" />
          {t("live.replies.dismiss")}
        </button>
      </div>
    </article>
  );
}
