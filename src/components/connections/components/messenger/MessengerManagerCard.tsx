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
    <div className="flex-1 min-w-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card flex flex-col justify-between overflow-hidden">
      {/* 1. Platform Identity & Status */}
      <div className="p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between gap-4 flex-wrap sm:flex-nowrap pb-1">
          <div className="flex items-center gap-3.5">
            <div
              className="w-[52px] h-[52px] rounded-[14.5px] flex items-center justify-center shrink-0 shadow-sm text-white"
              style={{ backgroundColor: messengerConfig.brandColor }}
            >
              <PlatformIcon
                platformId={PlatformId.MESSENGER}
                className="w-8 h-8 fill-current"
              />
            </div>

            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-3">
                <span className="text-[22px] font-semibold text-[#202733] leading-none">
                  {messengerConfig.name}
                </span>

                {/* Connection Status Badge */}
                {connectedCount > 0 ? (
                  <CommonBadge
                    color="success"
                    icon={
                      <span className="w-1.5 h-1.5 rounded-full bg-[#25805C]" />
                    }
                    status={`${connectedCount}/${totalAccounts} tài khoản kết nối`}
                  />
                ) : totalAccounts > 0 ? (
                  <CommonBadge
                    color="warning"
                    icon={
                      <span className="w-1.5 h-1.5 rounded-full bg-[#D97706]" />
                    }
                    status={`${totalAccounts} tài khoản (chưa đăng nhập)`}
                  />
                ) : (
                  <CommonBadge color="secondary" status="Chưa thêm tài khoản" />
                )}
              </div>
              <span className="text-[12px] text-[#6B7483]">
                {messengerConfig.tagline} • Hỗ trợ tối đa 5 tài khoản độc lập
              </span>
            </div>
          </div>
        </div>

        {/* 2. Account Rows List */}
        <div className="flex flex-col gap-2 pt-1">
          <div className="flex items-center justify-between">
            <span className="text-[13px] font-semibold text-[#202733]">
              Tài khoản liên kết
            </span>
            <span className="text-[11.5px] text-[#6B7483]">
              Độc lập phiên đăng nhập
            </span>
          </div>

          <MessengerAccountList />
        </div>

        {/* 3. WebView2 Architecture Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
          <div className="p-3.5 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/60 flex items-start gap-2.5">
            <Layers className="w-[17px] h-[17px] text-[#0866FF] shrink-0 mt-0.5" />
            <div className="flex flex-col gap-0.5">
              <span className="text-[11px] font-medium text-[#6B7483] tracking-wider uppercase">
                CƠ CHẾ PHÂN TÁCH DỮ LIỆU
              </span>
              <span className="text-[13px] font-medium text-[#202733]">
                {totalAccounts > 0
                  ? `${totalAccounts} Profile độc lập`
                  : "Chưa cấu hình"}
              </span>
              <span className="text-[11px] text-[#6B7483]">
                Lưu tại thư mục AppData riêng biệt
              </span>
            </div>
          </div>

          <div className="p-3.5 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/60 flex items-start gap-2.5">
            <RefreshCw className="w-[17px] h-[17px] text-[#0866FF] shrink-0 mt-0.5" />
            <div className="flex flex-col gap-0.5">
              <span className="text-[11px] font-medium text-[#6B7483] tracking-wider uppercase">
                ĐỒNG BỘ TIN CHƯA ĐỌC
              </span>
              <span className="text-[13px] font-medium text-[#202733]">
                {isMessengerConnected
                  ? "Thời gian thực (Realtime)"
                  : "Chưa có kết nối"}
              </span>
              <span className="text-[11px] text-[#6B7483]">
                Tự động gom số unread lên bong bóng
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* 4. Footer Note */}
      <div className="px-6 py-3.5 border-t border-[#E2E7EF] flex flex-wrap items-center justify-between gap-3 bg-white">
        <div className="flex items-center gap-2">
          <MonitorCheck className="w-4 h-4 text-[#25805C]" />
          <span className="text-[12px] font-medium text-[#202733]">
            Môi trường Native WebView2 an toàn & bảo mật
          </span>
        </div>

        <span className="text-[11.5px] text-[#6B7483]">
          Mỗi tab tài khoản hoạt động trong tiến trình riêng biệt
        </span>
      </div>
    </div>
  );
};
