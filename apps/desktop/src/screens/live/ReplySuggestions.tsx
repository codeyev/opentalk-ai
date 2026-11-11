import { useState } from "react";
import { DemoTag } from "../../components/DemoTag";
import { Icon } from "../../components/Icon";
import type { Replies, ReplyStyle } from "../../domain/conversation";
import { useI18n } from "../../i18n/I18nContext";
import { ReplyCard } from "./ReplyCard";

interface ReplySuggestionsProps {
  replies: Replies;
  /** BCP 47 tags of the languages the replies and their meanings are written in. */
  replyLanguage: string;
  meaningLanguage: string | null;
  isDemo: boolean;
  /** Asks for fresh suggestions; `null` when that is not possible right now. */
  onRefresh: (() => void) | null;
}

export function ReplySuggestions({
  replies,
  replyLanguage,
  meaningLanguage,
  isDemo,
  onRefresh,
}: ReplySuggestionsProps) {
  const { t } = useI18n();
  const noteId = "refresh-unavailable-note";

  return (
    <section className="panel" data-demo={isDemo} aria-labelledby="replies-title">
      <div className="panel-header">
        <h2 id="replies-title">
          {t("live.replies.title")} {isDemo && <DemoTag />}
        </h2>
        <button
          type="button"
          className="button button-quiet"
          disabled={onRefresh === null}
          aria-describedby={onRefresh === null ? noteId : undefined}
          onClick={onRefresh ?? undefined}
        >
          <Icon name="refresh" />
          {t("live.replies.refresh")}
        </button>
      </div>
      {onRefresh === null && (
        <p id={noteId} className="note">
          {t("live.replies.refreshUnavailable")}
        </p>
      )}

      <p role="status" className="note">
        {replies.status === "pending" && t("live.replies.pending")}
        {replies.status === "pending" &&
          replies.suggestions.length > 0 &&
          ` ${t("live.replies.outdated")}`}
      </p>
      {replies.status === "failed" && replies.failure !== null && (
        <p role="alert" className="error-text">
          {t("live.replies.failed", { reason: t(`live.reason.${replies.failure}`) })}
        </p>
      )}

      {/* Keyed by generation so edits and dismissals do not carry over to new suggestions. */}
      <ReplyList
        key={replies.generation}
        replies={replies}
        replyLanguage={replyLanguage}
        meaningLanguage={meaningLanguage}
      />
    </section>
  );
}

type ReplyListProps = Pick<ReplySuggestionsProps, "replies" | "replyLanguage" | "meaningLanguage">;

/** One set of suggestions, minus the ones the user dismissed. */
function ReplyList({ replies, replyLanguage, meaningLanguage }: ReplyListProps) {
  const { t } = useI18n();
  const [dismissed, setDismissed] = useState<readonly ReplyStyle[]>([]);
  const visible = replies.suggestions.filter((reply) => !dismissed.includes(reply.style));

  if (visible.length === 0) {
    return replies.status === "pending" ? null : <p className="empty">{t("live.replies.empty")}</p>;
  }
  return (
    <ul className="replies" data-outdated={replies.status === "pending"}>
      {visible.map((reply, index) => (
        <li key={reply.style}>
          <ReplyCard
            reply={reply}
            number={index + 1}
            replyLanguage={replyLanguage}
            meaningLanguage={meaningLanguage}
            onDismiss={() => {
              setDismissed([...dismissed, reply.style]);
            }}
          />
        </li>
      ))}
    </ul>
  );
}
