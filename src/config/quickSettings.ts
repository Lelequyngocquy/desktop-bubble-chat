export interface QuickSettingItem {
  id: string;
  labelKey: string;
  descriptionActiveKey: string;
  descriptionInactiveKey: string;
  iconName: "MessageSquare" | "Pin" | "Bell" | "Power";
  requiresConnection: boolean;
}

export const QUICK_SETTINGS_CONFIG: QuickSettingItem[] = [
  {
    id: "bubble",
    labelKey: "quick_settings.item_bubble_label",
    descriptionActiveKey: "quick_settings.item_bubble_active",
    descriptionInactiveKey: "quick_settings.item_bubble_inactive",
    iconName: "MessageSquare",
    requiresConnection: true,
  },
  {
    id: "always-on-top",
    labelKey: "quick_settings.item_always_on_top_label",
    descriptionActiveKey: "quick_settings.item_always_on_top_active",
    descriptionInactiveKey: "quick_settings.item_always_on_top_inactive",
    iconName: "Pin",
    requiresConnection: false,
  },
  {
    id: "notifications",
    labelKey: "quick_settings.item_notifications_label",
    descriptionActiveKey: "quick_settings.item_notifications_active",
    descriptionInactiveKey: "quick_settings.item_notifications_inactive",
    iconName: "Bell",
    requiresConnection: false,
  },
  {
    id: "autostart",
    labelKey: "quick_settings.item_autostart_label",
    descriptionActiveKey: "quick_settings.item_autostart_active",
    descriptionInactiveKey: "quick_settings.item_autostart_inactive",
    iconName: "Power",
    requiresConnection: false,
  },
];