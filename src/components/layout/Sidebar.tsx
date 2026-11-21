import React from "react";
import { useTranslation } from "react-i18next";
import {
  Home,
  Unplug,
  MessageSquare,
  Bell,
  Activity,
  CheckCircle2,
  Settings2,
  HelpCircle,
} from "lucide-react";
import { useNavigationStore } from "../../stores/navigationStore";
import {
  useMessengerStore,
  selectConnectedAccountCount,
} from "../../platforms/messenger";
import { useConversationStore } from "../../stores/conversationStore";
import { useSystemUserStore } from "../../stores/systemUserStore";
import { useUpdateStore } from "../../stores/updateStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { NavTab, NavigationItemConfig } from "../../types";
import { NAVIGATION_ITEMS } from "../../config";
import { AvatarCircle } from "../common";
import { cn } from "../../utils/cn";
import { SidebarNavItem } from "./sidebar/SidebarNavItem";
import { SidebarUpdateCard } from "./sidebar/SidebarUpdateCard";

export const Sidebar: React.FC = () => {
  const { t } = useTranslation(["common", "dashboard", "settings"]);
  const activeTab = useNavigationStore((s) => s.activeTab);
  const { setActiveTab, navigateToSection } = useNavigationStore.getState();
  const totalUnread = useConversationStore((s) =>
    s.conversations.reduce((acc, c) => acc + (c.unreadCount || 0), 0),
  );
  const connectedAccountCount = useMessengerStore(selectConnectedAccountCount);
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const currentLanguage = useSettingsStore((s) => s.language);
  const setLanguage = useSettingsStore.getState().setLanguage;
  const handleNavigateToUpdate = () => {
    navigateToSection("settings", "section-update");
  };
  const navIconMap: Record<NavigationItemConfig["iconName"], React.ReactNode> = {
    Home: <Home className="w-[18px] h-[18px]" />,
    Unplug: <Unplug className="w-[18px] h-[18px]" />,
    MessageSquare: <MessageSquare className="w-[18px] h-[18px]" />,
    Bell: <Bell className="w-[18px] h-[18px]" />,
    Activity: <Activity className="w-[18px] h-[18px]" />,
    Settings2: <Settings2 className="w-[18px] h-[18px]" />,
    HelpCircle: <HelpCircle className="w-[18px] h-[18px]" />,
  };
  const getBadgeForTab = (id: NavTab): number | undefined => {
    if (id === "connections") return connectedAccountCount;
    if (id === "notifications") return totalUnread > 0 ? totalUnread : undefined;
    return undefined;
  };
  const getNavLabel = (id: NavTab, fallback: string): string => {
    switch (id) {
      case "home":
        return t("dashboard:navigation.home");
      case "connections":
        return t("dashboard:navigation.connections");
      case "notifications":
        return t("dashboard:navigation.notifications");
      case "performance":
        return t("dashboard:navigation.performance");
      case "settings":
        return t("dashboard:navigation.settings");
      case "help":
        return t("dashboard:navigation.help");
      default:
        return fallback;
    }
  };
  const primaryNavItems = NAVIGATION_ITEMS.map((item) => ({
    id: item.id,
    label: getNavLabel(item.id, item.label),
    icon: navIconMap[item.iconName],
    badge: getBadgeForTab(item.id),
  }));
  const secondaryNavItems: Array<{ id: NavTab; label: string; icon: React.ReactNode }> = [
    { id: "settings", label: t("dashboard:navigation.settings"), icon: <Settings2 className="w-[18px] h-[18px]" /> },
    { id: "help", label: t("dashboard:navigation.help"), icon: <HelpCircle className="w-[18px] h-[18px]" /> },
  ];
  return (
    <aside className="w-sidebar shrink-0 bg-background-app border-r border-border flex flex-col justify-between p-[14px_12px_16px] select-none h-full overflow-hidden">
      <div className="flex flex-col gap-6">
        <button
          type="button"
          onClick={() => setActiveTab("home")}
          className="w-full text-left flex items-center gap-[11px] px-[10px] pb-2 cursor-pointer hover:opacity-90 transition-opacity"
        >
          <AvatarCircle size={42} text={systemUserInitials} className="shadow-xs" />
          <span className="flex flex-col gap-[2px] overflow-hidden min-w-0">
            <span className="text-nav font-semibold text-content-primary truncate leading-tight">
              {systemUser}
            </span>
            <span className="text-caption text-content-secondary truncate leading-tight">
              {t("common:personal_space")}
            </span>
          </span>
        </button>
        <nav className="flex flex-col gap-[5px]">
          {primaryNavItems.map((item) => (
            <SidebarNavItem
              key={item.id}
              label={item.label}
              icon={item.icon}
              badge={item.badge}
              isActive={activeTab === item.id}
              variant="primary"
              onClick={() => setActiveTab(item.id)}
            />
          ))}
        </nav>
      </div>
      <div className="flex flex-col gap-3">
        <SidebarUpdateCard onNavigateToUpdate={handleNavigateToUpdate} />
        <div className="bg-sidebar-card border border-sidebar-cardBorder rounded-panel p-[14px] flex flex-col gap-2">
          <div className="flex items-center gap-2 text-content-primary">
            <CheckCircle2 className="w-4 h-4 text-status-success" />
            <span className="text-small font-medium leading-none">
              {t("common:running_in_background")}
            </span>
          </div>
          <p className="text-caption text-content-secondary leading-snug m-0">
            {t("common:running_in_background_desc")}
          </p>
        </div>
        <div className="flex flex-col gap-1">
          {secondaryNavItems.map((item) => (
            <SidebarNavItem
              key={item.id}
              label={item.label}
              icon={item.icon}
              isActive={activeTab === item.id}
              variant="secondary"
              onClick={() => setActiveTab(item.id)}
            />
          ))}
        </div>
        <div className="flex items-center justify-center gap-1 pt-1 text-caption">
          <button
            type="button"
            aria-pressed={currentLanguage === "en"}
            onClick={() => setLanguage("en")}
            className={cn(
              "px-2 py-1 rounded transition-all cursor-pointer flex items-center gap-1.5",
              currentLanguage === "en"
                ? "bg-background-surface text-primary font-semibold shadow-xs"
                : "text-content-muted hover:text-content-primary hover:bg-background-subtle",
            )}
            title="Switch language to English"
          >
            <span>🇺🇸</span>
            <span>English</span>
          </button>
          <span className="text-border/70 select-none">/</span>
          <button
            type="button"
            aria-pressed={currentLanguage === "vi"}
            onClick={() => setLanguage("vi")}
            className={cn(
              "px-2 py-1 rounded transition-all cursor-pointer flex items-center gap-1.5",
              currentLanguage === "vi"
                ? "bg-background-surface text-primary font-semibold shadow-xs"
                : "text-content-muted hover:text-content-primary hover:bg-background-subtle",
            )}
            title="Switch language to Vietnamese"
          >
            <span>🇻🇳</span>
            <span>Tiếng Việt</span>
          </button>
        </div>
        <button
          type="button"
          onClick={handleNavigateToUpdate}
          className="pt-1 text-caption text-content-muted hover:text-primary cursor-pointer transition-colors flex items-center justify-center gap-1.5"
          title={t("settings:scroll_to_update")}
        >
          <span>DesktopBubbleChat v{currentVersion}</span>
        </button>
      </div>
    </aside>
  );
};

export default Sidebar;
