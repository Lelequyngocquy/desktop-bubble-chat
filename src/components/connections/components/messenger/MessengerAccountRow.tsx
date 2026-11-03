import React from "react";
import {
  ArrowUpRight,
  LogIn,
  RefreshCw,
  Trash2,
  CheckCircle2,
  AlertCircle,
  X,
  Activity,
  Moon,
} from "lucide-react";
import { MessengerAccount } from "../../../../types/account";
import { AvatarCircle, CommonButton, CommonBadge } from "../../../common";
import { getAccountThemeColor, Color, Dimen } from "../../../../constants/design";

interface MessengerAccountRowProps {
  account: MessengerAccount;
  onOpen: (account: MessengerAccount) => void;
  onRefresh: (account: MessengerAccount) => void;
  onRemove: (account: MessengerAccount) => void;
  onCancelAuth?: (account: MessengerAccount) => void;
  isRefreshing?: boolean;
}

/**
 * Component hiển thị thông tin 1 hàng tài khoản Messenger (Single Responsibility)
 * Màu ảnh đại diện được cấp từ bảng 5 màu đại diện riêng biệt theo số thứ tự (M1..M5).
 */
export const MessengerAccountRow: React.FC<MessengerAccountRowProps> = ({
  account,
  onOpen,
  onRemove,
  onCancelAuth,
}) => {
  const themeColor = getAccountThemeColor(account.index);

  return (
    <div className="flex items-center justify-between p-3.5 bg-background-subtle hover:bg-background-workspace rounded-card-sm border border-border/80 transition-all gap-4 flex-wrap sm:flex-nowrap">
      {/* Thông tin tài khoản */}
      <div className="flex items-center gap-3 min-w-0">
        {/* Avatar hiển thị màu sắc đại diện riêng biệt theo số thứ tự tài khoản */}
        <AvatarCircle
          size={Dimen.layout.avatarMd}
          text={account.avatarText}
          bg={themeColor.hex}
          textColor={Color.text.inverse}
          className="font-bold text-body shadow-xs shrink-0 ring-2 ring-white"
        />

        <div className="flex flex-col gap-0.5 overflow-hidden">
          <div className="flex items-center gap-2">
            <span className="text-nav font-semibold text-content-primary truncate">
              {account.name}
            </span>

            {/* Trạng thái kết nối của tài khoản */}
            {account.isConnected ? (
              <div className="flex items-center gap-1.5 shrink-0">
                {/* Trường hợp: Đã kết nối */}
                <CommonBadge
                  status="Đã kết nối"
                  icon={<CheckCircle2 />}
                  color="success"
                />

                {/* Trường hợp: Đang hoạt động / Ngủ đông */}
                {account.isBubbleVisible ? (
                  <CommonBadge
                    status="Đang hoạt động"
                    icon={<Activity />}
                    color="primary"
                  />
                ) : (
                  <CommonBadge
                    status="Đang ngủ đông"
                    icon={<Moon />}
                    color="secondary"
                    title="Tiến trình đã tắt hoàn toàn, 0 MB RAM, cache bảo toàn trên đĩa"
                  />
                )}
              </div>
            ) : account.isAuthenticating ? (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-primary-subtle text-primary text-caption font-medium rounded-pill shrink-0">
                <RefreshCw className="w-3 h-3 animate-spin text-primary" />
                Đang chờ đăng nhập...
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-status-warningBg text-status-warning text-caption font-medium rounded-pill shrink-0">
                <AlertCircle className="w-3 h-3 text-status-warning" />
                Chưa kết nối
              </span>
            )}
          </div>

          <span className="text-caption text-content-secondary truncate">
            {account.isConnected
              ? `Phiên hoạt động độc lập • ${account.lastSynced || "Đồng bộ vừa xong"}`
              : account.isAuthenticating
                ? "Đang mở cửa sổ đăng nhập riêng biệt. Bấm nút bên cạnh để đưa cửa sổ lên trước."
                : "Bấm 'Đăng nhập' để liên kết phiên WebView2 riêng"}
          </span>
        </div>
      </div>

      {/* Cụm hành động cho riêng tài khoản này */}
      <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
        {/* Nút Mở Messenger / Đăng nhập / Chờ đăng nhập */}
        {account.isAuthenticating ? (
          <div className="flex items-center gap-1.5">
            <CommonButton
              variant="secondary"
              size="sm"
              iconTrailing={
                <RefreshCw className="w-3.5 h-3.5 animate-spin text-primary" />
              }
              onClick={() => onOpen(account)}
              title="Bấm để đưa cửa sổ đăng nhập lên phía trước"
            >
              Xem cửa sổ
            </CommonButton>
            {onCancelAuth && (
              <button
                type="button"
                title="Hủy chờ đăng nhập"
                onClick={() => onCancelAuth(account)}
                className="p-2 rounded-control text-content-secondary hover:text-status-error hover:bg-status-errorBg border border-border transition-all cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        ) : (
          <CommonButton
            variant={account.isConnected ? "primary" : "secondary"}
            size="sm"
            iconTrailing={
              account.isConnected ? (
                <ArrowUpRight className="w-3.5 h-3.5" />
              ) : (
                <LogIn className="w-3.5 h-3.5 text-primary" />
              )
            }
            onClick={() => onOpen(account)}
          >
            {account.isConnected ? "Mở Messenger" : "Đăng nhập"}
          </CommonButton>
        )}

        {/* Nút Gỡ bỏ tài khoản */}
        <button
          type="button"
          title={`Gỡ bỏ ${account.name}`}
          onClick={() => onRemove(account)}
          className="p-2 rounded-control text-content-muted hover:text-status-error hover:bg-status-errorBg border border-transparent hover:border-status-errorBorder transition-all cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

export default MessengerAccountRow;
