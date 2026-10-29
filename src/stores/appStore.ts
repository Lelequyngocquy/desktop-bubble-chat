import { useNavigationStore } from "./navigationStore";
import {
  useMessengerStore,
  selectIsMessengerConnected,
  selectConnectedAccountSummary,
  selectTotalUnreadCount,
} from "./messengerStore";
import { useSettingsStore } from "./settingsStore";
import { useWindowStore } from "./windowStore";
import { useConversationStore } from "./conversationStore";
import { useSystemUserStore } from "./systemUserStore";
import { systemService } from "../services/systemService";
import { messengerService } from "../services/messenger";
import { NavTab } from "../types/navigation";
import { ConnectedAccount } from "../types/account";

export type { NavTab };

/**
 * Composite Facade Hook bridging dedicated domain stores.
 * Guarantees Single Source of Truth (SSOT) and clean separation while preserving
 * seamless backwards compatibility across the application.
 */
export function useAppStore() {
  const nav = useNavigationStore();
  const messenger = useMessengerStore();
  const settings = useSettingsStore();
  const win = useWindowStore();
  const conv = useConversationStore();
  const sys = useSystemUserStore();

  const isConnected = selectIsMessengerConnected(messenger);
  const connectedAccount = selectConnectedAccountSummary(messenger);
  const totalUnread = selectTotalUnreadCount(messenger);

  return {
    // System User
    systemUser: sys.systemUser,
    systemUserInitials: sys.systemUserInitials,
    fetchSystemUser: async () => {
      const username = await systemService.getSystemUsername();
      sys.setSystemUser(username);
    },

    // Navigation
    activeTab: nav.activeTab,
    setActiveTab: nav.setActiveTab,

    // Messenger Accounts (SSOT)
    messengerAccounts: messenger.accounts,
    activeAccountIndex: messenger.activeAccountIndex,
    setActiveAccountIndex: messenger.setActiveAccountIndex,
    addMessengerAccount: messenger.addAccount,
    removeMessengerAccount: async (id: string) => {
      const target = messenger.accounts.find((a) => a.id === id);
      messenger.removeAccount(id);
      if (target) {
        try {
          await messengerService.toggleBubble(false, target.index);
        } catch {}
      }
    },
    updateMessengerAccount: messenger.updateAccount,
    setMessengerAccounts: messenger.setAccounts,

    // Derived Platform State (Pure Computed - No State Duplication)
    isMessengerConnected: isConnected,
    setIsMessengerConnected: (connected: boolean) => {
      if (!connected) {
        messenger.disconnectAllAccounts();
        settings.setBubbleEnabled(false);
      }
    },
    connectedAccount,
    setConnectedAccount: (_account: ConnectedAccount | null) => {
      // Maintained via accounts SSOT
    },
    unreadCount: totalUnread,
    setUnreadCount: messenger.setTotalUnreadCount,

    // Window State
    isChatOpen: win.isChatOpen,
    setIsChatOpen: win.setIsChatOpen,

    // Settings
    isBubbleEnabled: settings.isBubbleEnabled,
    setBubbleEnabled: settings.setBubbleEnabled,
    toggleBubble: () => {
      if (!isConnected && !settings.isBubbleEnabled) {
        return false;
      }
      settings.toggleBubble();
      return true;
    },
    isAlwaysOnTop: settings.isAlwaysOnTop,
    toggleAlwaysOnTop: settings.toggleAlwaysOnTop,
    isNotificationsEnabled: settings.isNotificationsEnabled,
    toggleNotifications: settings.toggleNotifications,
    isAutoStart: settings.isAutoStart,
    setAutoStart: settings.setAutoStart,
    toggleAutoStart: settings.toggleAutoStart,

    // Conversations
    conversations: conv.conversations,
    setConversations: conv.setConversations,
    removeConversation: conv.removeConversation,
  };
}

// Static getter for non-react contexts (e.g. event listeners / background tasks)
useAppStore.getState = () => {
  const nav = useNavigationStore.getState();
  const messenger = useMessengerStore.getState();
  const settings = useSettingsStore.getState();
  const win = useWindowStore.getState();
  const conv = useConversationStore.getState();
  const sys = useSystemUserStore.getState();

  const isConnected = selectIsMessengerConnected(messenger);
  const connectedAccount = selectConnectedAccountSummary(messenger);
  const totalUnread = selectTotalUnreadCount(messenger);

  return {
    systemUser: sys.systemUser,
    systemUserInitials: sys.systemUserInitials,
    activeTab: nav.activeTab,
    setActiveTab: nav.setActiveTab,
    messengerAccounts: messenger.accounts,
    activeAccountIndex: messenger.activeAccountIndex,
    setActiveAccountIndex: messenger.setActiveAccountIndex,
    addMessengerAccount: messenger.addAccount,
    removeMessengerAccount: async (id: string) => {
      const target = messenger.accounts.find((a) => a.id === id);
      messenger.removeAccount(id);
      if (target) {
        try {
          await messengerService.toggleBubble(false, target.index);
        } catch {}
      }
    },
    updateMessengerAccount: messenger.updateAccount,
    setMessengerAccounts: messenger.setAccounts,
    isMessengerConnected: isConnected,
    setIsMessengerConnected: (connected: boolean) => {
      if (!connected) {
        messenger.disconnectAllAccounts();
        settings.setBubbleEnabled(false);
      }
    },
    connectedAccount,
    unreadCount: totalUnread,
    setUnreadCount: messenger.setTotalUnreadCount,
    isChatOpen: win.isChatOpen,
    setIsChatOpen: win.setIsChatOpen,
    isBubbleEnabled: settings.isBubbleEnabled,
    setBubbleEnabled: settings.setBubbleEnabled,
    toggleBubble: () => {
      if (!isConnected && !settings.isBubbleEnabled) {
        return false;
      }
      settings.toggleBubble();
      return true;
    },
    isAlwaysOnTop: settings.isAlwaysOnTop,
    toggleAlwaysOnTop: settings.toggleAlwaysOnTop,
    isNotificationsEnabled: settings.isNotificationsEnabled,
    toggleNotifications: settings.toggleNotifications,
    isAutoStart: settings.isAutoStart,
    setAutoStart: settings.setAutoStart,
    toggleAutoStart: settings.toggleAutoStart,
    conversations: conv.conversations,
    setConversations: conv.setConversations,
    removeConversation: conv.removeConversation,
  };
};
