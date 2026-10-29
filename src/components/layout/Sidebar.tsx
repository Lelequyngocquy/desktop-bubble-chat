import React from "react";
import {
  Home,
  Unplug,
  MessageSquare,
  Bell,
  CheckCircle2,
  Settings2,
  HelpCircle,
} from "lucide-react";
import { useNavigationStore } from "../../stores/navigationStore";
import { useMessengerStore, selectIsMessengerConnected } from "../../stores/messengerStore";
import { useConversationStore } from "../../stores/conversationStore";
import { useSystemUserStore } from "../../stores/systemUserStore";
import { NavTab, NavigationItemConfig } from "../../types";
import { MOCK_NAVIGATION_ITEMS } from "../../mocks";
import { AvatarCircle } from "../common";

export const Sidebar: React.FC = () => {
  const activeTab = useNavigationStore((s) => s.activeTab);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const conversations = useConversationStore((s) => s.conversations);
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);

  const navIconMap: Record<NavigationItemConfig["iconName"], React.ReactNode> = {
    Home: <Home className="w-[18px] h-[18px]" />,
    Unplug: <Unplug className="w-[18px] h-[18px]" />,
    MessageSquare: <MessageSquare className="w-[18px] h-[18px]" />,
    Bell: <Bell className="w-[18px] h-[18px]" />,
    Settings2: <Settings2 className="w-[18px] h-[18px]" />,
  };

  const getBadgeForTab = (id: NavTab): number | string | undefined => {
    switch (id) {
      case "connections":
        return isMessengerConnected ? 1 : 0;
      case "chat-heads":
        return conversations.length;
      default:
        return undefined;
    }
  };

  const primaryNavItems = MOCK_NAVIGATION_ITEMS.map((item) => ({
    id: item.id,
    label: item.label,
    icon: navIconMap[item.iconName],
    badge: getBadgeForTab(item.id),
  }));

  return (
    <aside className="w-[232px] shrink-0 bg-[#EEF2F9] border-r border-[#E2E7EF] flex flex-col justify-between p-[14px_12px_16px] select-none h-full overflow-hidden">
      {/* Top section: Profile & Primary Nav */}
      <div className="flex flex-col gap-6">
        {/* Profile - Windows User Profile */}
        <div
          onClick={() => setActiveTab("home")}
          className="flex items-center gap-[11px] px-[10px] pb-2 cursor-pointer hover:opacity-90 transition-opacity"
        >
          <AvatarCircle
            size={42}
            text={systemUserInitials}
            bg="#EAF2FF"
            textColor="#0759DB"
            className="shadow-xs"
          />
          <div className="flex flex-col gap-[2px] overflow-hidden">
            <span className="text-[14px] font-semibold text-[#202733] truncate leading-tight">
              {systemUser}
            </span>
            <span className="text-[11px] text-[#6B7483] truncate leading-tight">
              Không gian cá nhân
            </span>
          </div>
        </div>

        {/* Primary Navigation */}
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
                {/* Active selection indicator */}
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
                    className={`text-[14px] leading-tight transition-colors ${
                      isActive
                        ? "font-semibold text-[#0759DB]"
                        : "font-normal text-[#202733] group-hover:text-[#202733]"
                    }`}
                  >
                    {item.label}
                  </span>
                </div>

                {/* Badge if present */}
                {item.badge !== undefined && (
                  <span
                    className={`text-[12px] font-normal px-1.5 py-0.5 rounded-full ${
                      isActive
                        ? "text-[#0759DB]"
                        : "text-[#6B7483] group-hover:text-[#202733]"
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

      {/* Bottom section: Background service, Secondary Nav & Version */}
      <div className="flex flex-col gap-4">
        {/* Background Service Card */}
        <div className="bg-white/50 border border-white/75 rounded-[8px] p-[14px] flex flex-col gap-2">
          <div className="flex items-center gap-2 text-[#202733]">
            <CheckCircle2 className="w-4 h-4 text-[#25805C]" />
            <span className="text-[12px] font-medium leading-none">
              Đang chạy nền
            </span>
          </div>
          <p className="text-[11px] text-[#6B7483] leading-snug m-0">
            Đóng cửa sổ vẫn giữ chat heads hoạt động.
          </p>
        </div>

        {/* Secondary Navigation */}
        <div className="flex flex-col gap-1">
          <button
            onClick={() => setActiveTab("settings")}
            className="flex items-center gap-3 h-[38px] px-3 rounded-[6px] hover:bg-white/40 active:bg-white/60 transition-colors text-left text-[#202733]"
          >
            <Settings2 className="w-[18px] h-[18px] text-[#6B7483]" />
            <span className="text-[13px] font-normal">Cài đặt</span>
          </button>
          <button
            onClick={() => {}}
            className="flex items-center gap-3 h-[38px] px-3 rounded-[6px] hover:bg-white/40 active:bg-white/60 transition-colors text-left text-[#202733]"
          >
            <HelpCircle className="w-[18px] h-[18px] text-[#6B7483]" />
            <span className="text-[13px] font-normal">Trợ giúp & phản hồi</span>
          </button>
        </div>

        {/* App Version */}
        <div className="pt-2 pl-[42px] text-[11px] text-[#929AA7]">
          Phiên bản 1.0.0 · Windows 11
        </div>
      </div>
    </aside>
  );
};
