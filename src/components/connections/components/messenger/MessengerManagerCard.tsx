import React from "react";
import { MonitorCheck, RefreshCw, Layers } from "lucide-react";
import {
  useMessengerStore,
  selectIsMessengerConnected,
} from "../../../../platforms/messenger";
import { PlatformId } from "../../../../types/platform";
import { PLATFORM_CONFIGS } from "../../../../config";
import { PlatformIcon } from "../../../icons";
import { MessengerAccountList } from "./MessengerAccountList";
import { CommonBadge } from "../../../common";

export const MessengerManagerCard: React.FC = () => {
  const accounts = useMessengerStore((s) => s.accounts);
  const isMessengerConnected = useMessengerStore(selectIsMessengerConnected);
  const messengerConfig = PLATFORM_CONFIGS[PlatformId.MESSENGER];

  const connectedCount = accounts.filter((a) => a.isConnected).length;
  const totalAccounts = accounts.length;

  return (
    <div className="flex-1 min-w-0 bg-background-surface border border-border rounded-card shadow-card flex flex-col justify-between overflow-hidden">
      {/* 1. Platform Identity & Status */}
      <div className="p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between gap-4 flex-wrap sm:flex-nowrap pb-1">
          <div className="flex items-center gap-3.5">
            <div
              className="w-[52px] h-[52px] rounded-card flex items-center justify-center shrink-0 shadow-sm text-white"
              style={{ backgroundColor: messengerConfig.brandColor }}
            >
              <PlatformIcon
                platformId={PlatformId.MESSENGER}
                className="w-8 h-8 fill-current"
              />
            </div>

            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-3">
                <span className="text-platform font-semibold text-content-primary leading-none">
                  {messengerConfig.name}
                </span>

                {/* Connection Status Badge */}
                {connectedCount > 0 ? (
                  <CommonBadge
                    color="success"
                    icon={
                      <span className="w-1.5 h-1.5 rounded-pill bg-status-success" />
                    }
                    status={`${connectedCount}/${totalAccounts} tài khoản kết nối`}
                  />
                ) : totalAccounts > 0 ? (
                  <CommonBadge
                    color="warning"
                    icon={
                      <span className="w-1.5 h-1.5 rounded-pill bg-status-warning" />
                    }
                    status={`${totalAccounts} tài khoản (chưa đăng nhập)`}
                  />
                ) : (
                  <CommonBadge color="secondary" status="Chưa thêm tài khoản" />
                )}
              </div>
              <span className="text-small text-content-secondary">
                {messengerConfig.tagline} • Hỗ trợ tối đa 5 tài khoản độc lập
              </span>
            </div>
          </div>
        </div>

        {/* 2. Account Rows List */}
        <div className="flex flex-col gap-2 pt-1">
          <div className="flex items-center justify-between">
            <span className="text-body font-semibold text-content-primary">
              Tài khoản liên kết
            </span>
            <span className="text-caption text-content-secondary">
              Độc lập phiên đăng nhập
            </span>
          </div>

          <MessengerAccountList />
        </div>

        {/* 3. WebView2 Architecture Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
          <div className="p-3.5 bg-background-subtle rounded-panel border border-border/60 flex items-start gap-2.5">
            <Layers className="w-[17px] h-[17px] text-primary shrink-0 mt-0.5" />
            <div className="flex flex-col gap-0.5">
              <span className="text-caption font-medium text-content-secondary tracking-wider uppercase">
                CƠ CHẾ PHÂN TÁCH DỮ LIỆU
              </span>
              <span className="text-body font-medium text-content-primary">
                {totalAccounts > 0
                  ? `${totalAccounts} Profile độc lập`
                  : "Chưa cấu hình"}
              </span>
              <span className="text-caption text-content-secondary">
                Lưu tại thư mục AppData riêng biệt
              </span>
            </div>
          </div>

          <div className="p-3.5 bg-background-subtle rounded-panel border border-border/60 flex items-start gap-2.5">
            <RefreshCw className="w-[17px] h-[17px] text-primary shrink-0 mt-0.5" />
            <div className="flex flex-col gap-0.5">
              <span className="text-caption font-medium text-content-secondary tracking-wider uppercase">
                ĐỒNG BỘ TIN CHƯA ĐỌC
              </span>
              <span className="text-body font-medium text-content-primary">
                {isMessengerConnected
                  ? "Thời gian thực (Realtime)"
                  : "Chưa có kết nối"}
              </span>
              <span className="text-caption text-content-secondary">
                Tự động gom số unread lên bong bóng
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* 4. Footer Note */}
      <div className="px-6 py-3.5 border-t border-border flex flex-wrap items-center justify-between gap-3 bg-background-surface">
        <div className="flex items-center gap-2">
          <MonitorCheck className="w-4 h-4 text-status-success" />
          <span className="text-small font-medium text-content-primary">
            Môi trường Native WebView2 an toàn & bảo mật
          </span>
        </div>

        <span className="text-caption text-content-secondary">
          Mỗi tab tài khoản hoạt động trong tiến trình riêng biệt
        </span>
      </div>
    </div>
  );
};

export default MessengerManagerCard;
