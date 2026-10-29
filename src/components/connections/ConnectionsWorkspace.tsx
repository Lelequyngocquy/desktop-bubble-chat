import React from "react";
import { CheckCircle2, AlertCircle } from "lucide-react";
import { useMessengerStore, selectIsMessengerConnected } from "../../stores/messengerStore";
import { TabHeader } from "../common/TabHeader";
import {
  MessengerManagerCard,
  MessengerExplainerCard,
  MessengerTroubleshoot,
} from "./messenger";
import { PlatformRoadmap } from "../home/PlatformRoadmap";
import { ConnectionsFooter } from "./ConnectionsFooter";

export const ConnectionsWorkspace: React.FC = () => {
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const accounts = useMessengerStore((s) => s.accounts);
  const connectedCount = accounts.filter((a) => a.isConnected).length;

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-[#F4F7FC] overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="QUẢN LÝ NỀN TẢNG"
        title="Kết nối"
        description="Quản lý tài khoản, phiên đăng nhập và trạng thái đồng bộ của bạn."
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-white border border-[#E2E7EF] rounded-full text-[12px] shadow-xs">
            {isMessengerConnected ? (
              <>
                <CheckCircle2 className="w-3.5 h-3.5 text-[#25805C]" />
                <span className="text-[#202733] font-medium">
                  {connectedCount > 1 ? `${connectedCount} tài khoản kết nối` : "Đã kết nối"}
                </span>
              </>
            ) : (
              <>
                <AlertCircle className="w-3.5 h-3.5 text-[#D97706]" />
                <span className="text-[#6B7483] font-medium">
                  Chưa có kết nối
                </span>
              </>
            )}
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Tiêu đề mục nền tảng đã kết nối */}
        <div className="shrink-0 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-[16px] font-semibold text-[#202733] m-0">
              Nền tảng đã kết nối
            </h2>
            <span className="px-2 py-0.5 bg-[#EAF2FF] text-[#0759DB] text-[11px] font-semibold rounded-[6px]">
              {connectedCount > 0 ? connectedCount : 0}
            </span>
          </div>

          <div className="flex items-center gap-1.5 text-[12px] text-[#6B7483]">
            {isMessengerConnected ? (
              <>
                <CheckCircle2 className="w-[14px] h-[14px] text-[#25805C]" />
                <span>Mọi kết nối đang hoạt động</span>
              </>
            ) : (
              <>
                <AlertCircle className="w-[14px] h-[14px] text-[#D97706]" />
                <span>Chưa có kết nối nào</span>
              </>
            )}
          </div>
        </div>

        {/* Khối quản lý 2 cột */}
        <div className="shrink-0 flex flex-col xl:flex-row items-stretch gap-6">
          <MessengerManagerCard />
          <MessengerExplainerCard />
        </div>

        {/* Lộ trình nền tảng sắp tới */}
        <PlatformRoadmap />

        {/* Khối khắc phục sự cố */}
        <MessengerTroubleshoot />

        {/* Footer */}
        <ConnectionsFooter />
      </main>
    </div>
  );
};
