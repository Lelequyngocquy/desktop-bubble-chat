export type NavTab = "home" | "connections" | "chat-heads" | "notifications" | "updates" | "settings" | "help";

export interface NavigationItemConfig {
  id: NavTab;
  label: string;
  iconName: "Home" | "Unplug" | "MessageSquare" | "Bell" | "ArrowUpCircle" | "Settings2" | "HelpCircle";
}
