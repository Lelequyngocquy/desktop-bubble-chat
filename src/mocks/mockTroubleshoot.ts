export interface TroubleshootStep {
  num: number;
  title: string;
  desc: string;
}

export const MOCK_TROUBLESHOOT_STEPS: TroubleshootStep[] = [
  {
    num: 1,
    title: "Kiểm tra Internet",
    desc: "Đảm bảo máy tính đang có kết nối mạng ổn định.",
  },
  {
    num: 2,
    title: "Mở Messenger để kiểm tra",
    desc: "Đăng nhập lại nếu phiên xác thực trên thiết bị đã hết hạn.",
  },
  {
    num: 3,
    title: "Thử kết nối lại",
    desc: "Dùng chức năng Kết nối lại nếu dữ liệu tin nhắn chưa được cập nhật.",
  },
];
