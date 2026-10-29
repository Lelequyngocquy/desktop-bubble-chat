import React, { useState, useEffect } from "react";
import {
  ArrowUpRight,
  MonitorCheck,
  RefreshCw,
  LogIn,
} from "lucide-react";
import { messengerService } from "../../../services/messenger";
import { useAppStore } from "../../../stores/appStore";
import { AvatarCircle, CommonButton } from "../../common";
import { PlatformId } from "../../../types/platform";
import { MOCK_PLATFORMS, MOCK_DEFAULT_CONNECTED_ACCOUNT } from "../../../mocks";
import { PlatformIcon } from "../../icons";

export const MessengerManagerCard: React.FC = () => {
  const {
    isMessengerConnected,
    setIsMessengerConnected,
    connectedAccount,
    setConnectedAccount,
  } = useAppStore();

  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  const messengerConfig = MOCK_PLATFORMS[PlatformId.MESSENGER];

  useEffect(() => {
    messengerService.checkAuth()
      .then((res) => {
        if (res && res.connected) {
          const accountName = res.name && res.name.trim() ? res.name.trim() : MOCK_DEFAULT_CONNECTED_ACCOUNT.name;
          setIsMessengerConnected(true);
          setConnectedAccount({
            name: accountName,
            avatarText: MOCK_DEFAULT_CONNECTED_ACCOUNT.avatarText,
            lastSynced: "Đồng bộ vừa xong",
          });
        }
      })
      .catch(() => {});
  }, [setIsMessengerConnected, setConnectedAccount]);

  useEffect(() => {
    if (isMessengerConnected) {
      setIsAuthenticating(false);
    }
  }, [isMessengerConnected]);

  const handleOpenAuth = async () => {
    if (isMessengerConnected) {
      try {
        await messengerService.openChat();
      } catch {
        // Browser preview fallback
      }
      return;
    }

    setIsAuthenticating(true);
    try {
      await messengerService.openAuth();
      const pollTimer = setInterval(async () => {
        try {
          const res = await messengerService.checkAuth();
          if (res && res.connected) {
            clearInterval(pollTimer);
            setIsAuthenticating(false);
            const accountName = res.name && res.name.trim() ? res.name.trim() : MOCK_DEFAULT_CONNECTED_ACCOUNT.name;
            setIsMessengerConnected(true);
            setConnectedAccount({
              name: accountName,
              avatarText: MOCK_DEFAULT_CONNECTED_ACCOUNT.avatarText,
              lastSynced: "Đồng bộ vừa xong",
            });
          }
        } catch {}
      }, 1000);

      setTimeout(() => {
        clearInterval(pollTimer);
        setIsAuthenticating(false);
      }, 180000);
    } catch {
      setIsAuthenticating(false);
      setIsMessengerConnected(true);
      setConnectedAccount(MOCK_DEFAULT_CONNECTED_ACCOUNT);
    }
  };

  const handleRefreshConnection = async () => {
    setIsRefreshing(true);
    try {
      await messengerService.refresh();
      const res = await messengerService.checkAuth();
      if (res && res.connected) {
        const accountName = res.name && res.name.trim() ? res.name.trim() : MOCK_DEFAULT_CONNECTED_ACCOUNT.name;
        setIsMessengerConnected(true);
        setConnectedAccount({
          name: accountName,
          avatarText: MOCK_DEFAULT_CONNECTED_ACCOUNT.avatarText,
          lastSynced: "Đồng bộ vừa xong",
        });
      }
    } catch {
    } finally {
      setTimeout(() => setIsRefreshing(false), 800);
    }
  };

  const handleDisconnect = async () => {
    try {
      await messengerService.disconnect();
    } catch {}
    setIsMessengerConnected(false);
    setConnectedAccount(null);
  };

  return (
    <div className="flex-1 min-w-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card flex flex-col justify-between overflow-hidden">
      {/* Top Details */}
      <div className="p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between gap-4 flex-wrap sm:flex-nowrap">
          <div className="flex items-center gap-3.5">
            <div
              className="w-[52px] h-[52px] rounded-[14.5px] flex items-center justify-center shrink-0 shadow-sm text-white"
              style={{ backgroundColor: messengerConfig.brandColor }}
            >
              <PlatformIcon platformId={PlatformId.MESSENGER} className="w-8 h-8 fill-current" />
            </div>

            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-3">
                <span className="text-[22px] font-semibold text-[#202733] leading-none">
                  {messengerConfig.name}
                </span>
                {isMessengerConnected ? (
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 bg-[#E9F5EF] text-[#25805C] text-[11px] font-medium rounded-full">
                    <span className="w-1.5 h-1.5 rounded-full bg-[#25805C]" />
                    Đã kết nối
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 bg-[#FFF3E8] text-[#D97706] text-[11px] font-medium rounded-full">
                    <span className="w-1.5 h-1.5 rounded-full bg-[#D97706]" />
                    Chưa kết nối
                  </span>
                )}
              </div>
              <span className="text-[12px] text-[#6B7483]">
                {messengerConfig.tagline}
              </span>
            </div>
          </div>

          <CommonButton
            variant="primary"
            size="md"
            iconTrailing={
              isAuthenticating && !isMessengerConnected ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : isMessengerConnected ? (
                <ArrowUpRight className="w-4 h-4" />
              ) : (
                <LogIn className="w-4 h-4" />
              )
            }
            onClick={handleOpenAuth}
            disabled={isAuthenticating && !isMessengerConnected}
          >
            {isAuthenticating && !isMessengerConnected
              ? "Đang chờ đăng nhập..."
              : isMessengerConnected
                ? `Mở ${messengerConfig.name}`
                : `Đăng nhập ${messengerConfig.name}`}
          </CommonButton>
        </div>

        {/* Account Box */}
        <div className="flex items-center gap-2.5 p-3 rounded-[8px] bg-[#F8FAFD] border border-[#E2E7EF]/60">
          <AvatarCircle
            size={36}
            text={connectedAccount ? connectedAccount.avatarText : "—"}
            bg={isMessengerConnected ? "#EAF2FF" : "#EEF2F9"}
            textColor={isMessengerConnected ? "#0759DB" : "#6B7483"}
          />
          <div className="flex flex-col gap-0.5 overflow-hidden">
            <span className="text-[14px] font-semibold text-[#202733] truncate leading-tight">
              {connectedAccount
                ? connectedAccount.name
                : "Chưa có tài khoản nào được liên kết"}
            </span>
            <span className="text-[11px] text-[#6B7483] truncate leading-tight">
              {isMessengerConnected
                ? `Tài khoản ${messengerConfig.name} hiện tại`
                : `Bấm 'Đăng nhập ${messengerConfig.name}' để liên kết tài khoản`}
            </span>
          </div>
        </div>

        {/* Session Status 2-Column Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div className="p-3.5 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/60 flex items-start gap-2.5">
            <MonitorCheck className="w-[17px] h-[17px] text-[#0866FF] shrink-0 mt-0.5" />
            <div className="flex flex-col gap-0.5">
              <span className="text-[11px] font-medium text-[#6B7483] tracking-wider uppercase">
                PHIÊN ĐĂNG NHẬP
              </span>
              <span className="text-[13px] font-medium text-[#202733]">
                {isMessengerConnected ? "Đang hoạt động" : "Chưa có phiên"}
              </span>
              <span className="text-[11px] text-[#6B7483]">
                {isMessengerConnected
                  ? "Được lưu trên thiết bị này"
                  : "Yêu cầu đăng nhập để lưu phiên"}
              </span>
            </div>
          </div>

          <div className="p-3.5 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF]/60 flex items-start gap-2.5">
            <RefreshCw
              className={`w-[17px] h-[17px] text-[#0866FF] shrink-0 mt-0.5 ${
                isRefreshing ? "animate-spin" : ""
              }`}
            />
            <div className="flex flex-col gap-0.5">
              <span className="text-[11px] font-medium text-[#6B7483] tracking-wider uppercase">
                ĐỒNG BỘ
              </span>
              <span className="text-[13px] font-medium text-[#202733]">
                {isMessengerConnected ? "Đồng bộ vừa xong" : "Chưa đồng bộ"}
              </span>
              <span className="text-[11px] text-[#6B7483]">
                {isMessengerConnected
                  ? "Không cần thao tác thêm"
                  : "Tự động đồng bộ sau khi kết nối"}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Footer Actions */}
      <div className="px-6 py-3.5 border-t border-[#E2E7EF] flex flex-wrap items-center justify-between gap-3 bg-white">
        <div className="flex flex-col gap-0.5">
          <span className="text-[12px] font-medium text-[#202733]">
            Cần làm mới kết nối?
          </span>
          <span className="text-[11px] text-[#6B7483]">
            Thử kết nối lại nếu tin nhắn chưa cập nhật.
          </span>
        </div>

        <div className="flex items-center gap-2">
          <CommonButton
            variant="secondary"
            size="sm"
            iconLeading={
              <RefreshCw
                className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`}
              />
            }
            onClick={handleRefreshConnection}
            disabled={!isMessengerConnected || isRefreshing}
          >
            Kết nối lại
          </CommonButton>

          {isMessengerConnected && (
            <CommonButton variant="danger" size="sm" onClick={handleDisconnect}>
              Ngắt kết nối
            </CommonButton>
          )}
        </div>
      </div>
    </div>
  );
};
