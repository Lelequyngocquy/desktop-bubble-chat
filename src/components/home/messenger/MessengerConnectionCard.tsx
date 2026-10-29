import React from "react";
import { ArrowUpRight, Activity, ShieldCheck, LogIn } from "lucide-react";
import { messengerService } from "../../../services/messenger";
import { useAppStore } from "../../../stores/appStore";
import { PlatformId } from "../../../types";
import { MOCK_PLATFORMS } from "../../../mocks";
import { PlatformIcon } from "../../icons";
import {
  AvatarCircle,
  AvatarSquareRounded,
  CommonButton,
} from "../../common";

export const MessengerConnectionCard: React.FC = () => {
  const {
    unreadCount,
    conversations,
    setIsChatOpen,
    isMessengerConnected,
    connectedAccount,
    setActiveTab,
  } = useAppStore();

  const messengerConfig = MOCK_PLATFORMS[PlatformId.MESSENGER];

  const handleOpenMessenger = async () => {
    if (!isMessengerConnected) {
      // Chuyển sang tab Kết nối để xác thực
      setActiveTab("connections");
      return;
    }
    try {
      await messengerService.openChat();
      setIsChatOpen(true);
    } catch {
      // Browser preview fallback
      setIsChatOpen(true);
    }
  };

  return (
    <div className="shrink-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card overflow-hidden flex flex-col select-none">
      {/* Top Connection Details Area (Figma Node #3:1208) */}
      <div className="p-6 flex items-center justify-between gap-6 flex-wrap lg:flex-nowrap">
        {/* Left: Platform Identity & Account Actions */}
        <div className="flex flex-col gap-5 flex-1 min-w-0 sm:min-w-[320px]">
          {/* Platform Identity (Figma Node #3:1210) */}
          <div className="flex items-center gap-[14px]">
            {/* Messenger Logo */}
            <AvatarSquareRounded
              size={52}
              radius={14.5}
              bg={messengerConfig.brandColor}
              shadow
              icon={
                <PlatformIcon
                  platformId={PlatformId.MESSENGER}
                  size={34}
                  className="fill-white"
                />
              }
            />

            {/* Platform Details */}
            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-3">
                <span className="text-[22px] font-semibold text-[#202733] leading-none">
                  {messengerConfig.name}
                </span>
                {isMessengerConnected ? (
                  <div className="flex items-center gap-1.5 px-[10px] py-1 bg-[#EAF6EF] rounded-full">
                    <div className="w-1.5 h-1.5 rounded-full bg-[#25805C]" />
                    <span className="text-[12px] font-medium text-[#25805C] leading-none">
                      Đã kết nối
                    </span>
                  </div>
                ) : (
                  <div className="flex items-center gap-1.5 px-[10px] py-1 bg-[#FFF3E8] rounded-full">
                    <div className="w-1.5 h-1.5 rounded-full bg-[#D97706]" />
                    <span className="text-[12px] font-medium text-[#D97706] leading-none">
                      Chưa kết nối
                    </span>
                  </div>
                )}
              </div>
              <span className="text-[13px] text-[#6B7483]">
                {messengerConfig.tagline}
              </span>
            </div>
          </div>

          {/* Connected Account & Action buttons (Figma Node #3:1220) */}
          <div className="flex items-center justify-between gap-4 pt-1 flex-wrap sm:flex-nowrap">
            {/* Account Info */}
            <div className="flex items-center gap-2.5">
              <AvatarCircle
                size={32}
                text={connectedAccount ? connectedAccount.avatarText : "—"}
                bg={isMessengerConnected ? "#EAF2FF" : "#EEF2F9"}
                textColor={isMessengerConnected ? "#0759DB" : "#6B7483"}
              />
              <div className="flex flex-col">
                <span className="text-[13px] font-medium text-[#202733] leading-tight">
                  {connectedAccount ? connectedAccount.name : "Chưa liên kết tài khoản"}
                </span>
                <span className="text-[11px] text-[#6B7483] leading-tight">
                  {isMessengerConnected
                    ? "Đồng bộ vừa xong"
                    : "Đăng nhập tại tab Kết nối"}
                </span>
              </div>
            </div>

            {/* Action buttons */}
            <div className="flex items-center gap-2">
              <CommonButton
                variant="outline"
                onClick={() => setActiveTab("connections")}
              >
                Quản lý
              </CommonButton>
              <CommonButton
                variant="primary"
                onClick={handleOpenMessenger}
                iconTrailing={
                  isMessengerConnected ? (
                    <ArrowUpRight className="w-4 h-4" />
                  ) : (
                    <LogIn className="w-4 h-4" />
                  )
                }
              >
                {isMessengerConnected ? `Mở ${messengerConfig.name}` : "Kết nối ngay"}
              </CommonButton>
            </div>
          </div>
        </div>

        {/* Right: Metrics Column (Figma Node #3:1232) */}
        <div className="w-full lg:w-[250px] shrink-0 border-t lg:border-t-0 lg:border-l border-[#E2E7EF] pt-4 lg:pt-0 lg:pl-6 flex flex-col justify-center gap-3">
          <div className="flex items-center gap-1.5 text-[#6B7483]">
            <Activity className="w-[14px] h-[14px]" />
            <span className="text-[11px] font-medium tracking-wider uppercase">
              Trạng thái hiện tại
            </span>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col">
              <span className="text-[26px] font-bold text-[#202733] leading-none">
                {isMessengerConnected ? conversations.length : 0}
              </span>
              <span className="text-[12px] text-[#6B7483] mt-1">
                Chat heads đang bật
              </span>
            </div>

            <div className="flex flex-col">
              <span className="text-[26px] font-bold text-[#202733] leading-none">
                {isMessengerConnected ? unreadCount : 0}
              </span>
              <span className="text-[12px] text-[#6B7483] mt-1">
                Tin chưa đọc
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Reassurance Banner (Figma Node #3:1243) */}
      <div className="bg-[#EAF2FF] px-6 py-2.5 flex items-center gap-2 text-[#0759DB] border-t border-[#E2E7EF]/50">
        <ShieldCheck className="w-4 h-4 shrink-0" />
        <span className="text-[11px] font-normal leading-tight">
          Phiên đăng nhập được lưu trên thiết bị này. Không gửi dữ liệu tài khoản
          ra ngoài.
        </span>
      </div>
    </div>
  );
};
