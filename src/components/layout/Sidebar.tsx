import React from "react";
import {
  Home,
  Unplug,
  MessageSquare,
  Bell,
  CheckCircle2,
  Settings2,
  HelpCircle,
  ArrowUpCircle,
} from "lucide-react";
import { useNavigationStore } from "../../stores/navigationStore";
import {
  useMessengerStore,
  selectConnectedAccounts,
} from "../../platforms/messenger";
import { useConversationStore } from "../../stores/conversationStore";
import { useSystemUserStore } from "../../stores/systemUserStore";
import { useUpdateStore } from "../../stores/updateStore";
import { NavTab, NavigationItemConfig } from "../../types";
import { NAVIGATION_ITEMS } from "../../config";
import { AvatarCircle } from "../common";

export const Sidebar: React.FC = () => {
  const activeTab = useNavigationStore((s) => s.activeTab);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const conversations = useConversationStore((s) => s.conversations);
  const connectedAccounts = useMessengerStore(selectConnectedAccounts);
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);
  const updateStatus = useUpdateStore((s) => s.status);
  const currentVersion = useUpdateStore((s) => s.currentVersion);

  const hasUpdate =
    updateStatus === "available" ||
    updateStatus === "downloading" ||
    updateStatus === "downloaded";

  const navIconMap: Record<NavigationItemConfig["iconName"], React.ReactNode> =
    {
      Home: <Home className="w-[18px] h-[18px]" />,
      Unplug: <Unplug className="w-[18px] h-[18px]" />,
      MessageSquare: <MessageSquare className="w-[18px] h-[18px]" />,
      Bell: <Bell className="w-[18px] h-[18px]" />,
      ArrowUpCircle: <ArrowUpCircle className="w-[18px] h-[18px]" />,
      Settings2: <Settings2 className="w-[18px] h-[18px]" />,
      HelpCircle: <HelpCircle className="w-[18px] h-[18px]" />,
    };

  const getBadgeForTab = (id: NavTab): number | string | undefined => {
    switch (id) {
      case "connections":
        return connectedAccounts.length;
      case "chat-heads":
        return conversations.length;
      default:
        return undefined;
    }
  };

  const primaryNavItems = NAVIGATION_ITEMS.map((item) => ({
    id: item.id,
    label: item.label,
    icon: navIconMap[item.iconName],
    badge: getBadgeForTab(item.id),
  }));

  const secondaryNavItems: Array<{
    id: NavTab;
    label: string;
    icon: React.ReactNode;
    badge?: React.ReactNode;
  }> = [
    {
      id: "settings",
      label: "Cài đặt",
      icon: <Settings2 className="w-[18px] h-[18px]" />,
    },
    {
      id: "help",
      label: "Trợ giúp & phản hồi",
      icon: <HelpCircle className="w-[18px] h-[18px]" />,
    },
    {
      id: "updates",
      label: "Cập nhật",
      icon: <ArrowUpCircle className="w-[18px] h-[18px]" />,
      badge: hasUpdate ? (
        <span className="w-2 h-2 rounded-full bg-primary animate-pulse shadow-xs" />
      ) : undefined,
    },
  ];

  return (
    <aside className="w-sidebar shrink-0 bg-background-app border-r border-border flex flex-col justify-between p-[14px_12px_16px] select-none h-full overflow-hidden">
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
            className="shadow-xs"
          />
          <div className="flex flex-col gap-[2px] overflow-hidden">
            <span className="text-nav font-semibold text-content-primary truncate leading-tight">
              {systemUser}
            </span>
            <span className="text-caption text-content-secondary truncate leading-tight">
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
                className={`relative flex items-center justify-between h-[42px] px-3 rounded-control transition-all text-left group ${
                  isActive
                    ? "bg-white/70 shadow-sm"
                    : "hover:bg-white/40 active:bg-white/60"
                }`}
              >
                {/* Active selection indicator */}
                {isActive && (
                  <div className="absolute left-0 top-[13px] w-[3px] h-[16px] bg-primary rounded-indicator" />
                )}

                <div className="flex items-center gap-3">
                  <span
                    className={`transition-colors ${
                      isActive
                        ? "text-primary-foreground"
                        : "text-content-secondary group-hover:text-content-primary"
                    }`}
                  >
                    {item.icon}
                  </span>
                  <span
                    className={`text-nav leading-tight transition-colors ${
                      isActive
                        ? "font-semibold text-primary-foreground"
                        : "font-normal text-content-primary group-hover:text-content-primary"
                    }`}
                  >
                    {item.label}
                  </span>
                </div>

                {/* Badge if present */}
                {item.badge !== undefined && (
                  <span
                    className={`text-small font-normal px-1.5 py-0.5 rounded-pill ${
                      isActive
                        ? "text-primary-foreground"
                        : "text-content-secondary group-hover:text-content-primary"
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
        <div className="bg-white/50 border border-white/75 rounded-panel p-[14px] flex flex-col gap-2">
          <div className="flex items-center gap-2 text-content-primary">
            <CheckCircle2 className="w-4 h-4 text-status-success" />
            <span className="text-small font-medium leading-none">
              Đang chạy nền
            </span>
          </div>
          <p className="text-caption text-content-secondary leading-snug m-0">
            Đóng cửa sổ vẫn sẽ giữ ứng dụng hoạt động.
          </p>
        </div>

        {/* Secondary Navigation */}
        <div className="flex flex-col gap-1">
          {secondaryNavItems.map((item) => {
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id)}
                className={`relative flex items-center justify-between h-[38px] px-3 rounded-control transition-all text-left group ${
                  isActive
                    ? "bg-white/70 shadow-sm"
                    : "hover:bg-white/40 active:bg-white/60"
                }`}
              >
                {/* Active selection indicator */}
                {isActive && (
                  <div className="absolute left-0 top-[11px] w-[3px] h-[16px] bg-primary rounded-indicator" />
                )}

                <div className="flex items-center gap-3">
                  <span
                    className={`transition-colors ${
                      isActive
                        ? "text-primary-foreground"
                        : "text-content-secondary group-hover:text-content-primary"
                    }`}
                  >
                    {item.icon}
                  </span>
                  <span
                    className={`text-body leading-tight transition-colors ${
                      isActive
                        ? "font-semibold text-primary-foreground"
                        : "font-normal text-content-primary group-hover:text-content-primary"
                    }`}
                  >
                    {item.label}
                  </span>
                </div>

                {/* Badge if present */}
                {item.badge}
              </button>
            );
          })}
        </div>

        {/* App Version */}
        <div
          onClick={() => setActiveTab("updates")}
          className="pt-2 pl-[42px] text-caption text-content-muted hover:text-primary cursor-pointer transition-colors"
          title="Bấm để mở trang cập nhật ứng dụng"
        >
          Phiên bản {currentVersion} · Windows 11
        </div>
      </div>
    </aside>
  );
};

export default Sidebar;
