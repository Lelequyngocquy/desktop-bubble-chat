export interface QuickSettingItem {
  id: string;
  label: string;
  descriptionActive: string;
  descriptionInactive: string;
  iconName: "MessageSquare" | "Pin" | "Bell" | "Power";
  requiresConnection: boolean;
}

export const MOCK_QUICK_SETTINGS_SCHEMA: QuickSettingItem[] = [
  {
    id: "bubble",
    label: "Bong bóng nổi",
    descriptionActive: "Hiển thị chat heads trên desktop",
    descriptionInactive: "Cần kết nối Messenger để sử dụng",
    iconName: "MessageSquare",
    requiresConnection: true,
  },
  {
    id: "always-on-top",
    label: "Luôn nổi trên cùng",
    descriptionActive: "Giữ phía trên các cửa sổ khác",
    descriptionInactive: "Giữ phía trên các cửa sổ khác",
    iconName: "Pin",
    requiresConnection: false,
  },
  {
    id: "notifications",
    label: "Thông báo",
    descriptionActive: "Nhận thông báo khi có tin nhắn mới",
    descriptionInactive: "Nhận thông báo khi có tin nhắn mới",
    iconName: "Bell",
    requiresConnection: false,
  },
  {
    id: "autostart",
    label: "Khởi động cùng Windows",
    descriptionActive: "Sẵn sàng ngay khi bạn đăng nhập",
    descriptionInactive: "Sẵn sàng ngay khi bạn đăng nhập",
    iconName: "Power",
    requiresConnection: false,
  },
];
