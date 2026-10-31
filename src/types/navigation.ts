export type NavTab = "home" | "connections" | "chat-heads" | "notifications" | "settings" | "help";

export interface NavigationItemConfig {
  id: NavTab;
  label: string;
  iconName: "Home" | "Unplug" | "MessageSquare" | "Bell" | "Settings2" | "HelpCircle";
}
