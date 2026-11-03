import React, { useState } from "react";
import { HelpCircle, Send, CheckCircle2 } from "lucide-react";
import { CommonLargeCard } from "../../common/index";

export const FeedbackSection: React.FC = () => {
  const [feedbackText, setFeedbackText] = useState("");
  const [feedbackSent, setFeedbackSent] = useState(false);

  const handleSendFeedback = (e: React.FormEvent) => {
    e.preventDefault();
    if (!feedbackText.trim()) return;
    setFeedbackSent(true);
    setFeedbackText("");
    setTimeout(() => {
      setFeedbackSent(false);
    }, 4000);
  };

  return (
    <CommonLargeCard
      icon={<HelpCircle className="w-4 h-4 text-primary" />}
      title="Gửi phản hồi cho nhà phát triển"
    >
      <form onSubmit={handleSendFeedback} className="flex flex-col gap-3">
        <span className="text-small text-content-secondary">
          Bạn gặp sự cố hiển thị hoặc muốn đề xuất thêm nền tảng mới (Zalo,
          Telegram, Discord)? Hãy gửi tin nhắn cho chúng tôi:
        </span>

        <textarea
          value={feedbackText}
          onChange={(e) => setFeedbackText(e.target.value)}
          placeholder="Nhập nội dung phản hồi hoặc ý kiến đóng góp của bạn..."
          rows={3}
          className="w-full p-3 text-body border border-border rounded-panel focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary resize-none"
        />

        <div className="flex items-center justify-between">
          {feedbackSent ? (
            <div className="flex items-center gap-1.5 text-status-success text-small font-medium">
              <CheckCircle2 className="w-4 h-4" />
              <span>Cảm ơn bạn! Phản hồi đã được ghi nhận.</span>
            </div>
          ) : (
            <span className="text-caption text-content-muted">
              Phiên bản ứng dụng: v0.1.0 · DesktopBubbleChat
            </span>
          )}

          <button
            type="submit"
            disabled={!feedbackText.trim()}
            className="px-4 py-2 bg-primary hover:bg-primary-foreground disabled:opacity-40 disabled:cursor-not-allowed text-content-inverse text-small font-medium rounded-panel flex items-center gap-1.5 transition-colors cursor-pointer"
          >
            <Send className="w-3.5 h-3.5" />
            <span>Gửi phản hồi</span>
          </button>
        </div>
      </form>
    </CommonLargeCard>
  );
};

export default FeedbackSection;
