import React from "react";
import { ArrowUpRight, Activity, ShieldCheck } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import {
  AvatarCircle,
  AvatarSquareRounded,
  CommonButton,
} from "../common";

export const MessengerConnectionCard: React.FC = () => {
  const { unreadCount, conversations, setIsChatOpen } = useAppStore();

  const handleOpenMessenger = async () => {
    try {
      const isOpen = await invoke<boolean>("toggle_chat");
      setIsChatOpen(isOpen);
    } catch {
      // In standard browser dev preview
      setIsChatOpen(true);
    }
  };

  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card overflow-hidden flex flex-col select-none">
      {/* Top Connection Details Area (Figma Node #3:1208) */}
      <div className="p-6 flex items-center justify-between gap-6 flex-wrap lg:flex-nowrap">
        {/* Left: Platform Identity & Account Actions */}
        <div className="flex flex-col gap-5 flex-1 min-w-[320px]">
          {/* Platform Identity (Figma Node #3:1210) */}
          <div className="flex items-center gap-[14px]">
            {/* Messenger Logo */}
            <AvatarSquareRounded
              size={52}
              radius={14.5}
              bg="#0866FF"
              shadow
              icon={
                <svg
                  viewBox="0 0 36 36"
                  className="w-[34px] h-[34px] fill-white"
                >
                  <path d="M18 1C8.61 1 1 8.16 1 17c0 5.04 2.5 9.54 6.43 12.45V35l5.29-2.91c1.68.47 3.46.72 5.28.72 9.39 0 17-7.16 17-16S27.39 1 18 1zm1.75 21.62l-4.34-4.63-8.48 4.63 9.33-9.91 4.45 4.63 8.37-4.63-9.33 9.91z" />
                </svg>
              }
            />

            {/* Platform Details */}
            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-3">
                <span className="text-[22px] font-semibold text-[#202733] leading-none">
                  Messenger
                </span>
                <div className="flex items-center gap-1.5 px-[10px] py-1 bg-[#EAF6EF] rounded-full">
                  <div className="w-1.5 h-1.5 rounded-full bg-[#25805C]" />
                  <span className="text-[12px] font-medium text-[#25805C] leading-none">
                    Đã kết nối
                  </span>
                </div>
              </div>
              <span className="text-[13px] text-[#6B7483]">
                Kết nối trực tiếp qua Smart WebView Wrapper
              </span>
            </div>
          </div>

          {/* Connected Account & Action buttons (Figma Node #3:1220) */}
          <div className="flex items-center justify-between gap-4 pt-1 flex-wrap sm:flex-nowrap">
            {/* Account Info */}
            <div className="flex items-center gap-2.5">
              <AvatarCircle
                size={32}
                text="MA"
                bg="#EAF2FF"
                textColor="#0759DB"
              />
              <div className="flex flex-col">
                <span className="text-[13px] font-medium text-[#202733] leading-tight">
                  Minh Anh
                </span>
                <span className="text-[11px] text-[#6B7483] leading-tight">
                  Đồng bộ vừa xong
                </span>
              </div>
            </div>

            {/* Action buttons */}
            <div className="flex items-center gap-2">
              <CommonButton variant="outline">
                Quản lý
              </CommonButton>
              <CommonButton
                variant="primary"
                onClick={handleOpenMessenger}
                iconTrailing={<ArrowUpRight className="w-4 h-4" />}
              >
                Mở Messenger
              </CommonButton>
            </div>
          </div>
        </div>

        {/* Section Divider (Figma Node #3:1233) */}
        <div className="hidden lg:block w-[1px] h-[112px] bg-[#E2E7EF] shrink-0" />

        {/* Right Overview Metrics Box (Figma Node #3:1234) */}
        <div className="w-full lg:w-[276px] bg-[#F8FAFD] border border-[#E2E7EF]/60 rounded-[8px] p-[18px] flex flex-col gap-[14px] shrink-0">
          <div className="flex items-center gap-[7px] text-[#6B7483]">
            <Activity className="w-[15px] h-[15px]" />
            <span className="text-[12px] font-medium text-[#6B7483]">
              Trạng thái hiện tại
            </span>
          </div>

          <div className="flex items-center justify-between gap-6">
            <div className="flex flex-col gap-0.5">
              <span className="text-[28px] font-semibold text-[#202733] leading-none">
                {conversations.length}
              </span>
              <span className="text-[11px] text-[#6B7483] whitespace-nowrap">
                Chat heads đang bật
              </span>
            </div>

            <div className="flex flex-col gap-0.5">
              <span className="text-[28px] font-semibold text-[#202733] leading-none">
                {unreadCount}
              </span>
              <span className="text-[11px] text-[#6B7483] whitespace-nowrap">
                Tin chưa đọc
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Reassurance Banner (Figma Node #3:1245) */}
      <div className="h-10 px-6 bg-[#EAF2FF] border-t border-[#0866FF]/10 flex items-center gap-2">
        <ShieldCheck className="w-[15px] h-[15px] text-[#0759DB] shrink-0" />
        <span className="text-[12px] text-[#0759DB] truncate">
          Phiên đăng nhập được lưu trên thiết bị. Bạn có thể ngắt kết nối bất cứ lúc nào trong Quản lý.
        </span>
      </div>
    </div>
  );
};
