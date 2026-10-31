import React, { useState } from "react";
import {
  HelpCircle,
  Keyboard,
  MousePointer,
  Send,
  CheckCircle2,
  Sparkles,
} from "lucide-react";
import { TabHeader } from "../common";

export const HelpWorkspace: React.FC = () => {
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
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="TRỢ GIÚP & HỖ TRỢ"
        title="Trợ giúp & phản hồi"
        description="Mẹo sử dụng các phím tắt, điều hướng bong bóng nổi và gửi phản hồi đóng góp ý kiến."
        rightSlot={
          <span className="px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full text-[12px] font-semibold text-[#0759DB] shadow-xs">
            Bản xem trước (Preview)
          </span>
        }
      />

      {/* 2. Nội dung cuộn mượt mà */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Section 1: Hướng dẫn & Phím tắt */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <Sparkles className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Mẹo sử dụng nhanh
            </h2>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-4 rounded-xl border border-[#E2E7EF] bg-[#F8FAFC] flex items-start gap-3">
              <div className="w-9 h-9 rounded-lg bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
                <MousePointer className="w-5 h-5" />
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-[13px] font-semibold text-[#1E293B]">
                  Rê chuột 0.1s (Hover)
                </span>
                <span className="text-[11px] text-[#64748B] leading-relaxed">
                  Rê chuột vào bong bóng bất kỳ trong 100ms để mở ngay khung chat mà không cần click. Di chuột ra ngoài 1 giây khung chat sẽ tự động đóng êm ái.
                </span>
              </div>
            </div>

            <div className="p-4 rounded-xl border border-[#E2E7EF] bg-[#F8FAFC] flex items-start gap-3">
              <div className="w-9 h-9 rounded-lg bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
                <Keyboard className="w-5 h-5" />
              </div>
              <div className="flex flex-col gap-1">
                <div className="flex items-center gap-2">
                  <span className="text-[13px] font-semibold text-[#1E293B]">
                    Phím tắt toàn cục
                  </span>
                  <span className="px-1.5 py-0.5 bg-white border border-[#CBD5E1] rounded text-[10px] font-mono text-[#334155] font-semibold">
                    Ctrl + Shift + M
                  </span>
                </div>
                <span className="text-[11px] text-[#64748B] leading-relaxed">
                  Bấm tổ hợp phím này bất cứ lúc nào khi đang làm việc trên Windows để ẩn hoặc hiện toàn bộ bong bóng chat trên màn hình.
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Section 2: Gửi phản hồi & Đóng góp ý kiến */}
        <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
          <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
            <HelpCircle className="w-4 h-4 text-[#0866FF]" />
            <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
              Gửi phản hồi cho nhà phát triển
            </h2>
          </div>

          <form onSubmit={handleSendFeedback} className="flex flex-col gap-3">
            <span className="text-[12px] text-[#64748B]">
              Bạn gặp sự cố hiển thị hoặc muốn đề xuất thêm nền tảng mới (Zalo, Telegram, Discord)? Hãy gửi tin nhắn cho chúng tôi:
            </span>

            <textarea
              value={feedbackText}
              onChange={(e) => setFeedbackText(e.target.value)}
              placeholder="Nhập nội dung phản hồi hoặc ý kiến đóng góp của bạn..."
              rows={3}
              className="w-full p-3 text-[13px] border border-[#E2E7EF] rounded-lg focus:outline-none focus:border-[#0866FF] focus:ring-1 focus:ring-[#0866FF] resize-none"
            />

            <div className="flex items-center justify-between">
              {feedbackSent ? (
                <div className="flex items-center gap-1.5 text-green-600 text-[12px] font-medium">
                  <CheckCircle2 className="w-4 h-4" />
                  <span>Cảm ơn bạn! Phản hồi đã được ghi nhận.</span>
                </div>
              ) : (
                <span className="text-[11px] text-[#94A3B8]">
                  Phiên bản ứng dụng: v0.1.0 · DesktopBubbleChat
                </span>
              )}

              <button
                type="submit"
                disabled={!feedbackText.trim()}
                className="px-4 py-2 bg-[#0866FF] hover:bg-[#0759DB] disabled:opacity-40 disabled:cursor-not-allowed text-white text-[12px] font-medium rounded-lg flex items-center gap-1.5 transition-colors cursor-pointer"
              >
                <Send className="w-3.5 h-3.5" />
                <span>Gửi phản hồi</span>
              </button>
            </div>
          </form>
        </div>
      </main>
    </div>
  );
};

export default HelpWorkspace;
