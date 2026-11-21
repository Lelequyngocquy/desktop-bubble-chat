import React, { useState, useRef, useEffect } from "react";
import { HelpCircle, Send, CheckCircle2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CommonLargeCard } from "../../common/index";
import { useUpdateStore } from "../../../stores/updateStore";
import { systemService } from "../../../services/systemService";

const GITHUB_ISSUES_URL = "https://github.com/Lelequyngocquy/desktop-bubble-chat/issues/new";

export const FeedbackSection: React.FC = () => {
  const { t, i18n } = useTranslation("help");
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const [feedbackText, setFeedbackText] = useState("");
  const [feedbackSent, setFeedbackSent] = useState(false);
  const sentTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => {
    return () => {
      clearTimeout(sentTimerRef.current);
    };
  }, []);
  const handleSendFeedback = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = feedbackText.trim();
    if (!trimmed) return;
    const firstLine = trimmed.split(/\r?\n/)[0].slice(0, 72);
    const title = `[Feedback] ${firstLine}`;
    const body = [
      trimmed,
      "",
      "---",
      `App Version: v${currentVersion || "unknown"}`,
      `Language: ${i18n.language}`,
      `Platform: ${typeof navigator !== "undefined" ? navigator.userAgent : "Desktop"}`,
    ].join("\n");
    const issueUrl = `${GITHUB_ISSUES_URL}?title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
    await systemService.openExternalUrl(issueUrl);
    setFeedbackSent(true);
    setFeedbackText("");
    clearTimeout(sentTimerRef.current);
    sentTimerRef.current = setTimeout(() => {
      setFeedbackSent(false);
    }, 5000);
  };
  return (
    <CommonLargeCard
      icon={<HelpCircle className="w-4 h-4 text-primary" />}
      title={t("feedback_title")}
    >
      <form onSubmit={handleSendFeedback} className="flex flex-col gap-3">
        <span className="text-small text-content-secondary">
          {t("feedback_desc")}
        </span>
        <textarea
          value={feedbackText}
          onChange={(e) => setFeedbackText(e.target.value)}
          placeholder={t("feedback_placeholder")}
          rows={3}
          className="w-full p-3 text-body bg-background-subtle text-content-primary placeholder:text-content-muted border border-border rounded-panel focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary resize-none"
        />
        <div className="flex items-center justify-between gap-3">
          {feedbackSent ? (
            <div className="flex items-center gap-1.5 text-status-success text-small font-medium">
              <CheckCircle2 className="w-4 h-4 shrink-0" />
              <span>{t("feedback_thanks")}</span>
            </div>
          ) : (
            <span className="text-caption text-content-muted">
              {currentVersion ? t("app_version", { version: currentVersion }) : ""}
            </span>
          )}
          <button
            type="submit"
            disabled={!feedbackText.trim()}
            className="px-4 py-2 bg-primary hover:bg-primary-foreground disabled:opacity-40 disabled:cursor-not-allowed text-content-inverse text-small font-medium rounded-panel flex items-center gap-1.5 transition-colors cursor-pointer shrink-0"
          >
            <Send className="w-3.5 h-3.5" />
            <span>{t("send_btn")}</span>
          </button>
        </div>
      </form>
    </CommonLargeCard>
  );
};

export default FeedbackSection;
