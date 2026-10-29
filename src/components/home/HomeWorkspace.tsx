import React from "react";
import { MonitorCheck } from "lucide-react";
import { useAppStore } from "../../stores/appStore";
import { TabHeader } from "../common/TabHeader";
import { MessengerConnectionCard } from "./MessengerConnectionCard";
import { ActiveChatHeads } from "./ActiveChatHeads";
import { QuickSettings } from "./QuickSettings";
import { PlatformRoadmap } from "./PlatformRoadmap";
import { HomeFooter } from "./HomeFooter";

export const HomeWorkspace: React.FC = () => {
  const { systemUser } = useAppStore();

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="KHÔNG GIAN CỦA BẠN"
        title={`Chào ${systemUser},`}
        description="Mọi kết nối ở đây. Mọi cuộc trò chuyện luôn trong tầm tay."
        rightSlot={
          <div className="flex items-center gap-2 px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full shadow-xs">
            <MonitorCheck className="w-3.5 h-3.5 text-[#25805C]" />
            <span className="text-[12px] text-[#202733] font-medium">
              Sẵn sàng trên desktop
            </span>
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Thẻ Hero kết nối Messenger */}
        <MessengerConnectionCard />

        {/* Khối 2 cột: Chat heads & Thiết lập nhanh */}
        <div className="shrink-0 flex flex-col lg:flex-row items-stretch gap-6">
          <ActiveChatHeads />
          <QuickSettings />
        </div>

        {/* Lộ trình nền tảng sắp tới */}
        <PlatformRoadmap />

        {/* Footer */}
        <HomeFooter />
      </main>
    </div>
  );
};
