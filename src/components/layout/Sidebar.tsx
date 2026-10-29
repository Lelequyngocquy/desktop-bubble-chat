import React, { useState } from "react";
import {
  Home,
  Unplug,
  MessageSquare,
  Bell,
  CheckCircle2,
  Settings2,
  HelpCircle,
} from "lucide-react";
import { useAppStore } from "../../stores/appStore";
import { AvatarCircle } from "../common";

type NavTab = "home" | "connections" | "chat-heads" | "notifications";

export const Sidebar: React.FC = () => {
  const [activeTab, setActiveTab] = useState<NavTab>("home");
  const { conversations } = useAppStore();

  const primaryNavItems: {
    id: NavTab;
    label: string;
    icon: React.ReactNode;
    badge?: number | string;
  }[] = [
    {
      id: "home",
      label: "Home",
      icon: <Home className="w-[18px] h-[18px]" />,
    },
    {
      id: "connections",
      label: "Kết nối",
      icon: <Unplug className="w-[18px] h-[18px]" />,
      badge: 1,
    },
    {
      id: "chat-heads",
      label: "Chat heads",
      icon: <MessageSquare className="w-[18px] h-[18px]" />,
      badge: conversations.length,
    },
    {
      id: "notifications",
      label: "Thông báo",
      icon: <Bell className="w-[18px] h-[18px]" />,
    },
  ];

  return (
    <aside className="w-[232px] shrink-0 bg-[#EEF2F9] border-r border-[#E2E7EF] flex flex-col justify-between p-[14px_12px_16px] select-none h-full overflow-hidden">
      {/* Top section: Profile & Primary Nav */}
      <div className="flex flex-col gap-6">
        {/* Profile (Figma Node #3:1160) */}
        <div className="flex items-center gap-[11px] px-[10px] pb-2 cursor-pointer hover:opacity-90 transition-opacity">
          <AvatarCircle
            size={42}
            text="MA"
            bg="#EAF2FF"
            textColor="#0759DB"
            className="shadow-xs"
          />
          <div className="flex flex-col gap-[2px] overflow-hidden">
            <span className="text-[14px] font-semibold text-[#202733] truncate leading-tight">
              Minh Anh
            </span>
            <span className="text-[11px] text-[#6B7483] truncate leading-tight">
              Không gian cá nhân
            </span>
          </div>
        </div>

        {/* Primary Navigation (Figma Node #3:1166) */}
        <nav className="flex flex-col gap-[5px]">
          {primaryNavItems.map((item) => {
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id)}
                className={`relative flex items-center justify-between h-[42px] px-3 rounded-[6px] transition-all text-left group ${
                  isActive
                    ? "bg-white/70 shadow-sm"
                    : "hover:bg-white/40 active:bg-white/60"
                }`}
              >
                {/* Active selection indicator (Figma Node #3:1168) */}
                {isActive && (
                  <div className="absolute left-0 top-[13px] w-[3px] h-[16px] bg-[#0866FF] rounded-[2px]" />
                )}

                <div className="flex items-center gap-3">
                  <span
                    className={`transition-colors ${
                      isActive
                        ? "text-[#0759DB]"
                        : "text-[#6B7483] group-hover:text-[#202733]"
                    }`}
                  >
                    {item.icon}
                  </span>
                  <span
                    className={`text-[14px] transition-colors ${
                      isActive
                        ? "font-semibold text-[#0759DB]"
                        : "font-normal text-[#202733]"
                    }`}
                  >
                    {item.label}
                  </span>
                </div>

                {item.badge !== undefined && (
                  <span
                    className={`text-[12px] px-1.5 py-0.5 rounded-full ${
                      isActive
                        ? "text-[#0759DB] font-medium"
                        : "text-[#6B7483] font-normal"
                    }`}
                  >
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Bottom section: Background service info & Secondary Nav */}
      <div className="flex flex-col gap-4">
        {/* Background Service Status Card (Figma Node #3:1183) */}
        <div className="p-[14px] bg-white/50 border border-white/75 rounded-[8px] flex flex-col gap-2 shadow-sm backdrop-blur-xs">
          <div className="flex items-center gap-2 text-[#202733]">
            <CheckCircle2 className="w-4 h-4 text-[#25805C] shrink-0" />
            <span className="text-[12px] font-medium leading-none">
              Đang chạy nền
            </span>
          </div>
          <p className="text-[11px] text-[#6B7483] leading-[1.45em] m-0">
            Đóng cửa sổ vẫn giữ chat heads hoạt động.
          </p>
        </div>

        {/* Secondary Navigation (Figma Node #3:1188) */}
        <div className="flex flex-col gap-1">
          <button className="flex items-center gap-3 h-[38px] px-3 rounded-[6px] text-[#202733] hover:bg-white/40 active:bg-white/60 transition-colors text-left">
            <Settings2 className="w-[18px] h-[18px] text-[#6B7483]" />
            <span className="text-[13px] font-normal">Cài đặt</span>
          </button>

          <button className="flex items-center gap-3 h-[38px] px-3 rounded-[6px] text-[#202733] hover:bg-white/40 active:bg-white/60 transition-colors text-left">
            <HelpCircle className="w-[18px] h-[18px] text-[#6B7483]" />
            <span className="text-[13px] font-normal">Trợ giúp & phản hồi</span>
          </button>

          {/* App Version Info (Figma Node #3:1195) */}
          <div className="pt-2 pl-[42px]">
            <span className="text-[11px] text-[#929AA7] tracking-tight">
              Phiên bản 1.0.0 · Windows 11
            </span>
          </div>
        </div>
      </div>
    </aside>
  );
};
