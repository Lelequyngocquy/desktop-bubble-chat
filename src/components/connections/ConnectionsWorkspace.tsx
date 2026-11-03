import React from "react";
import { CheckCircle2, AlertCircle } from "lucide-react";
import {
  useMessengerStore,
  selectIsMessengerConnected,
} from "../../platforms/messenger";
import { TabHeader } from "../common/TabHeader";
import {
  MessengerManagerCard,
  MessengerTroubleshoot,
  ConnectionsFooter,
} from "./components";
import { PlatformRoadmap } from "../home";

export const ConnectionsWorkspace: React.FC = () => {
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const accounts = useMessengerStore((s) => s.accounts);
  const connectedCount = accounts.filter((a) => a.isConnected).length;

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      {/* 1. Header cố định chung */}
      <TabHeader
        eyebrow="QUẢN LÝ KẾT NỐI"
        title="Kết nối"
        description="Quản lý tài khoản, phiên đăng nhập và trạng thái đồng bộ của bạn."
        rightSlot={
          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
            {isMessengerConnected ? (
              <>
                <CheckCircle2 className="w-3.5 h-3.5 text-status-success" />
                <span className="text-content-primary font-medium">
                  {connectedCount > 1
                    ? `${connectedCount} tài khoản đã liên kết`
                    : "1 tài khoản đã liên kết"}
                </span>
              </>
            ) : (
              <>
                <AlertCircle className="w-3.5 h-3.5 text-status-warning" />
                <span className="text-content-secondary font-medium">
                  Chưa có kết nối
                </span>
              </>
            )}
          </div>
        }
      />

      {/* 2. Nội dung cuộn mượt mà luôn bung hết chiều rộng */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Khối quản lý 2 cột */}
        <div className="shrink-0 flex flex-col xl:flex-row items-stretch gap-6">
          <MessengerManagerCard />
        </div>

        {/* Khối khắc phục sự cố */}
        <MessengerTroubleshoot />

        {/* Lộ trình nền tảng sắp tới */}
        <PlatformRoadmap />

        {/* Footer */}
        <ConnectionsFooter />
      </main>
    </div>
  );
};

export default ConnectionsWorkspace;
