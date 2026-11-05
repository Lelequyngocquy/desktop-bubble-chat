export type NavTab = "home" | "connections" | "chat-heads" | "notifications" | "performance" | "settings" | "help";

export interface NavigationItemConfig {
  id: NavTab;
  label: string;
  iconName: "Home" | "Unplug" | "MessageSquare" | "Bell" | "Activity" | "Settings2" | "HelpCircle";
}
