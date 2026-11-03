import React from "react";
import { Settings } from "lucide-react";
import { PlatformId } from "../../../../types/platform";
import { PLATFORM_CONFIGS } from "../../../../config";
import { PlatformIcon } from "../../../icons";
import {
  AvatarSquareRounded,
  CommonBadge,
  CommonButton,
} from "../../../common";

interface MessengerPlatformHeaderProps {
  totalConnected: number;
  totalAccounts: number;
  areAnyBubblesVisible: boolean;
  onManage: () => void;
}

export const MessengerPlatformHeader: React.FC<
  MessengerPlatformHeaderProps
> = ({ totalConnected, totalAccounts, onManage }) => {
  const messengerConfig = PLATFORM_CONFIGS[PlatformId.MESSENGER];
  const isConnected = totalConnected > 0;

  return (
    <div className="flex items-center justify-between gap-4 flex-wrap pb-1">
      {/* Left: Platform Logo & Title */}
      <div className="flex items-center gap-[14px]">
        <AvatarSquareRounded
          size={52}
          radius={14}
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
            <span className="text-platform font-semibold text-content-primary leading-none">
              {messengerConfig.name}
            </span>

            {isConnected ? (
              <CommonBadge
                color="success"
                icon={
                  <span className="w-1.5 h-1.5 rounded-pill bg-status-success" />
                }
                status={
                  totalConnected === 1
                    ? "Đã kết nối"
                    : `${totalConnected}/${totalAccounts} tài khoản kết nối`
                }
              />
            ) : (
              <CommonBadge
                color="warning"
                icon={
                  <span className="w-1.5 h-1.5 rounded-pill bg-status-warning" />
                }
                status="Chưa kết nối"
              />
            )}
          </div>
          <span className="text-body text-content-secondary">
            {messengerConfig.tagline}
          </span>
        </div>
      </div>

      {/* Right: Master Actions Toolbar */}
      <div className="flex items-center gap-2">
        <CommonButton
          variant="outline"
          size="sm"
          onClick={onManage}
          iconLeading={<Settings className="w-3.5 h-3.5 text-content-secondary" />}
        >
          Quản lý
        </CommonButton>
      </div>
    </div>
  );
};

export default MessengerPlatformHeader;
