import React from "react";
import { TabHeader } from "../common";
import {
  QuickTipsSection,
  FeedbackSection,
  MessengerExplainerCard,
} from "./components";

export const HelpWorkspace: React.FC = () => {
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
        <QuickTipsSection />
        <MessengerExplainerCard />
        <FeedbackSection />
      </main>
    </div>
  );
};

export default HelpWorkspace;
