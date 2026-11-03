import React from "react";
import { Cpu, Sparkles, XCircle, MousePointerClick, Menu, Info } from "lucide-react";
import { CommonLargeCard } from "../../common/index";

export const ResourceOptimizationCard: React.FC = () => {
  return (
    <CommonLargeCard
      icon={<Cpu className="w-4 h-4 text-[#0866FF]" />}
      title="Gợi ý tối ưu tài nguyên: Chạy ẩn Dashboard"
      subTitle={
        <span className="px-2.5 py-0.5 bg-[#ECFDF5] text-[#059669] border border-[#A7F3D0] rounded-full text-[11px] font-semibold inline-flex items-center gap-1 shadow-2xs">
          <Sparkles className="w-3 h-3 text-[#059669]" />
          Khuyên dùng
        </span>
      }
      className="w-full shrink-0 p-6 justify-between gap-5 select-none"
    >
      <div className="flex flex-col gap-4">
        {/* Banner tóm tắt lợi ích tối ưu */}
        <div className="p-3.5 bg-[#F0FDF4] rounded-[10px] border border-[#BBF7D0] flex items-start gap-3">
          <div className="w-8 h-8 rounded-lg bg-[#DCFCE7] text-[#16A34A] flex items-center justify-center shrink-0 mt-0.5">
            <Cpu className="w-4 h-4" />
          </div>
          <div className="flex flex-col gap-1 text-[12px] text-[#166534] leading-relaxed">
            <span className="font-semibold text-[#14532D]">
              Giải phóng RAM & CPU khi không cần xem cài đặt
            </span>
            <span>
              Sau khi đã bật các bong bóng chat, bạn nên <strong>bấm nút [X] để đóng Dashboard</strong>.
              Giao diện trang quản lý sẽ được giải phóng hoàn toàn khỏi bộ nhớ, trong khi các Bong bóng Win32 siêu nhẹ
              vẫn duy trì kết nối nhận tin nhắn và thông báo liên tục 24/7.
            </span>
          </div>
        </div>

        {/* 3 Khối hướng dẫn chi tiết cách làm việc với Khay hệ thống (Tray Icon) */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-[12px] text-[#475467] leading-relaxed">
          {/* Bước 1 */}
          <div className="p-3.5 bg-[#F8FAFD] rounded-[10px] border border-[#E2E7EF] flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-md bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
                <XCircle className="w-4 h-4" />
              </div>
              <span className="font-semibold text-[#1E293B]">
                1. Đóng ẩn Dashboard
              </span>
            </div>
            <span className="text-[11px] text-[#64748B]">
              Nhấn nút <strong>[X]</strong> ở góc trên bên phải cửa sổ. Ứng dụng <strong>không bị thoát</strong> mà tự động thu nhỏ về chạy ngầm dưới Khay hệ thống (System Tray).
            </span>
          </div>

          {/* Bước 2 */}
          <div className="p-3.5 bg-[#F8FAFD] rounded-[10px] border border-[#E2E7EF] flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-md bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
                <MousePointerClick className="w-4 h-4" />
              </div>
              <span className="font-semibold text-[#1E293B]">
                2. Mở lại Dashboard
              </span>
            </div>
            <span className="text-[11px] text-[#64748B]">
              Khi cần thay đổi cấu hình, <strong>nhấp đúp chuột trái (Double-click)</strong> vào biểu tượng ứng dụng ở góc phải thanh Taskbar (gần đồng hồ Windows) để mở lại giao diện.
            </span>
          </div>

          {/* Bước 3 */}
          <div className="p-3.5 bg-[#F8FAFD] rounded-[10px] border border-[#E2E7EF] flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-md bg-[#EAF2FF] text-[#0759DB] flex items-center justify-center shrink-0">
                <Menu className="w-4 h-4" />
              </div>
              <span className="font-semibold text-[#1E293B]">
                3. Menu Khay hệ thống
              </span>
            </div>
            <span className="text-[11px] text-[#64748B]">
              <strong>Nhấp chuột phải</strong> vào biểu tượng khay để bật/tắt nhanh từng bong bóng, ẩn tất cả bong bóng khi cần tập trung, hoặc chọn <em>&quot;Thoát ứng dụng&quot;</em> để dừng hoàn toàn.
            </span>
          </div>
        </div>
      </div>

      {/* Ghi chú chân trang */}
      <div className="p-3 bg-[#EEF2F9]/60 rounded-[8px] flex items-start gap-2 text-[11px] text-[#6B7483]">
        <Info className="w-3.5 h-3.5 shrink-0 text-[#0866FF] mt-0.5" />
        <span>
          <strong>Mẹo tìm biểu tượng:</strong> Nếu không thấy biểu tượng ứng dụng trên Taskbar, hãy bấm vào mũi tên <strong>^ (Show hidden icons)</strong> ở góc phải khay hệ thống Windows.
        </span>
      </div>
    </CommonLargeCard>
  );
};
