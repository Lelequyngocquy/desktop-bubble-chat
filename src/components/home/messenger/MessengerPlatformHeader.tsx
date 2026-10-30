import React from "react";
import { Eye, EyeOff, Settings } from "lucide-react";
import { PlatformId } from "../../../types/platform";
import { PLATFORM_CONFIGS } from "../../../config";
import { PlatformIcon } from "../../icons";
import { AvatarSquareRounded, CommonButton } from "../../common";

interface MessengerPlatformHeaderProps {
  totalConnected: number;
  totalAccounts: number;
  areAnyBubblesVisible: boolean;
  onToggleAllBubbles: () => void;
  onManage: () => void;
}

export const MessengerPlatformHeader: React.FC<MessengerPlatformHeaderProps> = ({
  totalConnected,
  totalAccounts,
  areAnyBubblesVisible,
  onToggleAllBubbles,
  onManage,
}) => {
  const messengerConfig = PLATFORM_CONFIGS[PlatformId.MESSENGER];
  const isConnected = totalConnected > 0;

  return (
    <div className="flex items-center justify-between gap-4 flex-wrap pb-1">
      {/* Left: Platform Logo & Title */}
      <div className="flex items-center gap-[14px]">
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

        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-3">
            <span className="text-[22px] font-semibold text-[#202733] leading-none">
              {messengerConfig.name}
            </span>

            {isConnected ? (
              <div className="flex items-center gap-1.5 px-[10px] py-1 bg-[#EAF6EF] rounded-full">
                <div className="w-1.5 h-1.5 rounded-full bg-[#25805C]" />
                <span className="text-[12px] font-medium text-[#25805C] leading-none">
                  {totalConnected === 1
                    ? "Đã kết nối"
                    : `${totalConnected}/${totalAccounts} tài khoản kết nối`}
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

      {/* Right: Master Actions Toolbar */}
      <div className="flex items-center gap-2">
        {isConnected && (
          <CommonButton
            variant="outline"
            size="sm"
            onClick={onToggleAllBubbles}
            iconLeading={
              areAnyBubblesVisible ? (
                <EyeOff className="w-3.5 h-3.5 text-[#6B7483]" />
              ) : (
                <Eye className="w-3.5 h-3.5 text-[#0759DB]" />
              )
            }
            title={areAnyBubblesVisible ? "Ẩn toàn bộ bong bóng nổi" : "Hiện toàn bộ bong bóng nổi"}
          >
            {areAnyBubblesVisible ? "Ẩn tất cả" : "Hiện tất cả"}
          </CommonButton>
        )}

        <CommonButton
          variant="outline"
          size="sm"
          onClick={onManage}
          iconLeading={<Settings className="w-3.5 h-3.5 text-[#6B7483]" />}
        >
          Quản lý
        </CommonButton>
      </div>
    </div>
  );
};
