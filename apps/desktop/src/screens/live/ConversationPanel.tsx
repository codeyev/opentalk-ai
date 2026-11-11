import { DemoTag } from "../../components/DemoTag";
import { Icon } from "../../components/Icon";
import type { Conversation, TranscriptSegment } from "../../domain/conversation";
import { useI18n } from "../../i18n/I18nContext";
import { languageDisplayName } from "../../lib/language";
import { formatOffset } from "../../lib/time";
import { SegmentTranslation } from "./SegmentTranslation";

interface ConversationPanelProps {
  conversation: Pick<Conversation, "segments" | "translations">;
  isDemo: boolean;
}

/** What was said, oldest first, each with its translation underneath. */
export function ConversationPanel({ conversation, isDemo }: ConversationPanelProps) {
  const { t } = useI18n();
  const { segments, translations } = conversation;

  return (
    <section className="panel" data-demo={isDemo} aria-labelledby="conversation-title">
      <h2 id="conversation-title">
        {t("live.conversation.title")} {isDemo && <DemoTag />}
      </h2>
      {segments.length === 0 ? (
        <p className="empty">{t("live.transcript.empty")}</p>
      ) : (
        <ol className="segments">
          {segments.map((segment) => (
            <li key={segment.id} className="segment" data-source={segment.source}>
              <Segment segment={segment} />
              <SegmentTranslation state={translations[segment.id]} />
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

/** One thing that was said: who, in which language, when, and the words. */
function Segment({ segment }: { segment: TranscriptSegment }) {
  const { t, locale } = useI18n();
  return (
    <div className="segment-body" data-partial={!segment.isFinal}>
      <p className="segment-meta">
        <span className="segment-source">
          <Icon name={segment.source === "microphone" ? "microphone" : "monitor"} />
          {t(`live.source.${segment.source}`)}
        </span>
        {segment.language !== null && <span>{languageDisplayName(segment.language, locale)}</span>}
        {!segment.isFinal && <span className="partial-label">{t("live.segment.partial")}</span>}
        <time className="segment-time">{formatOffset(segment.startMs)}</time>
      </p>
      <p className="segment-text" lang={segment.language ?? undefined} dir="auto">
        {segment.text}
        {!segment.isFinal && "…"}
      </p>
    </div>
  );
}
