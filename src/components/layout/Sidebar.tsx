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
  Sparkles,
} from "lucide-react";
import { useNavigationStore } from "../../stores/navigationStore";
import {
  useMessengerStore,
  selectConnectedAccounts,
} from "../../platforms/messenger";
import { useConversationStore } from "../../stores/conversationStore";
import { useSystemUserStore } from "../../stores/systemUserStore";
import { useUpdateStore } from "../../stores/updateStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { NavTab, NavigationItemConfig } from "../../types";
import { NAVIGATION_ITEMS } from "../../config";
import { AvatarCircle } from "../common";

export const Sidebar: React.FC = () => {
  const { t } = useTranslation(["common", "dashboard", "settings"]);
  const activeTab = useNavigationStore((s) => s.activeTab);
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);
  const conversations = useConversationStore((s) => s.conversations);
  const connectedAccounts = useMessengerStore(selectConnectedAccounts);
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);
  const updateStatus = useUpdateStore((s) => s.status);
  const currentVersion = useUpdateStore((s) => s.currentVersion);
  const availableVersion = useUpdateStore((s) => s.availableVersion);
  const currentLanguage = useSettingsStore((s) => s.language);
  const setLanguage = useSettingsStore((s) => s.setLanguage);
  const hasUpdate =
    updateStatus === "available" ||
    updateStatus === "downloading" ||
    updateStatus === "downloaded";

  const handleNavigateToUpdate = () => {
    setActiveTab("settings");
    setTimeout(() => {
      const section = document.getElementById("section-update");
      if (section) {
        section.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 100);
  };
  const navIconMap: Record<NavigationItemConfig["iconName"], React.ReactNode> =
    {
      Home: <Home className="w-[18px] h-[18px]" />,
      Unplug: <Unplug className="w-[18px] h-[18px]" />,
      MessageSquare: <MessageSquare className="w-[18px] h-[18px]" />,
      Bell: <Bell className="w-[18px] h-[18px]" />,
      Activity: <Activity className="w-[18px] h-[18px]" />,
      Settings2: <Settings2 className="w-[18px] h-[18px]" />,
      HelpCircle: <HelpCircle className="w-[18px] h-[18px]" />,
    };
  const totalUnread = conversations.reduce(
    (acc, c) => acc + (c.unreadCount || 0),
    0,
  );
  const getBadgeForTab = (id: NavTab): number | string | undefined => {
    switch (id) {
      case "connections":
        return connectedAccounts.length;
      case "notifications":
        return totalUnread > 0 ? totalUnread : undefined;
      default:
        return undefined;
    }
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
  const secondaryNavItems: Array<{
    id: NavTab;
    label: string;
    icon: React.ReactNode;
    badge?: React.ReactNode;
  }> = [
    {
      id: "settings",
      label: t("dashboard:navigation.settings"),
      icon: <Settings2 className="w-[18px] h-[18px]" />,
    },
    {
      id: "help",
      label: t("dashboard:navigation.help"),
      icon: <HelpCircle className="w-[18px] h-[18px]" />,
    },
  ];
  return (
    <aside className="w-sidebar shrink-0 bg-background-app border-r border-border flex flex-col justify-between p-[14px_12px_16px] select-none h-full overflow-hidden">
      {/* Top section: Profile & Primary Nav */}
      <div className="flex flex-col gap-6">
        {/* Profile - User Header */}
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
              {t("common:personal_space")}
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
                    ? "bg-white/70 dark:bg-white/10 shadow-sm"
                    : "hover:bg-white/40 dark:hover:bg-white/5 active:bg-white/60 dark:active:bg-white/10"
                }`}
              >
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

      {/* Bottom section: Background service, Secondary Nav, Language Switcher & Version */}
      <div className="flex flex-col gap-3">
        {hasUpdate && (
          <div
            role="button"
            tabIndex={0}
            onClick={handleNavigateToUpdate}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                handleNavigateToUpdate();
              }
            }}
            className="bg-sidebar-card border border-sidebar-cardBorder hover:border-primary/50 dark:hover:border-primary/50 rounded-panel p-[14px] flex flex-col gap-2 cursor-pointer transition-all shadow-xs group"
            title={t("settings:scroll_to_update")}
          >
            <div className="flex items-center justify-between text-content-primary">
              <div className="flex items-center gap-2 min-w-0">
                <Sparkles className="w-4 h-4 text-primary shrink-0" />
                <span className="text-small font-medium leading-none truncate">
                  {t("common:update_widget_title")}
                </span>
              </div>
              <span className="px-1.5 py-0.5 bg-primary text-white text-[10px] font-bold rounded-pill animate-pulse leading-none shadow-xs shrink-0">
                (new)
              </span>
            </div>
            <p className="text-caption text-content-secondary leading-snug m-0 group-hover:text-content-primary transition-colors">
              {availableVersion
                ? t("common:update_widget_desc_ver", {
                    version: availableVersion,
                  })
                : t("common:update_widget_desc")}
            </p>
          </div>
        )}

        {/* Background Service Status Card */}
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
                    ? "bg-white/70 dark:bg-white/10 shadow-sm"
                    : "hover:bg-white/40 dark:hover:bg-white/5 active:bg-white/60 dark:active:bg-white/10"
                }`}
              >
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
                {item.badge}
              </button>
            );
          })}
        </div>

        {/* Quick Language Switcher */}
        <div className="flex items-center justify-center gap-1 pt-1 text-caption">
          <button
            type="button"
            onClick={() => setLanguage("en")}
            className={`px-2 py-1 rounded transition-all cursor-pointer flex items-center gap-1.5 ${
              currentLanguage === "en"
                ? "bg-background-surface text-primary font-semibold shadow-xs"
                : "text-content-muted hover:text-content-primary hover:bg-background-subtle"
            }`}
            title="Switch language to English"
          >
            <span>🇺🇸</span>
            <span>English</span>
          </button>
          <span className="text-border/70 select-none">/</span>
          <button
            type="button"
            onClick={() => setLanguage("vi")}
            className={`px-2 py-1 rounded transition-all cursor-pointer flex items-center gap-1.5 ${
              currentLanguage === "vi"
                ? "bg-background-surface text-primary font-semibold shadow-xs"
                : "text-content-muted hover:text-content-primary hover:bg-background-subtle"
            }`}
            title="Switch language to Vietnamese"
          >
            <span>🇻🇳</span>
            <span>Tiếng Việt</span>
          </button>
        </div>

        {/* App Version */}
        <div
          onClick={handleNavigateToUpdate}
          className="pt-1 text-caption text-content-muted hover:text-primary cursor-pointer transition-colors flex items-center justify-center gap-1.5"
          title="Open Settings & Updates"
        >
          <span>DesktopBubbleChat v{currentVersion}</span>
        </div>
      </div>
    </aside>
  );
};

export default Sidebar;
