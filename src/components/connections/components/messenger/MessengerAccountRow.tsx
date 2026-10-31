import React from "react";
import {
  ArrowUpRight,
  LogIn,
  RefreshCw,
  Trash2,
  CheckCircle2,
  AlertCircle,
  X,
} from "lucide-react";
import { MessengerAccount } from "../../../../types/account";
import { AvatarCircle, CommonButton } from "../../../common";
import { getAccountThemeColor } from "../../../../constants/accountColors";

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
  // onRefresh,
  onRemove,
  onCancelAuth,
  // isRefreshing = false,
}) => {
  const themeColor = getAccountThemeColor(account.index);

  return (
    <div className="flex items-center justify-between p-3.5 bg-[#F8FAFD] hover:bg-[#F3F7FD] rounded-[10px] border border-[#E2E7EF]/80 transition-all gap-4 flex-wrap sm:flex-nowrap">
      {/* Thông tin tài khoản */}
      <div className="flex items-center gap-3 min-w-0">
        {/* Avatar hiển thị màu sắc đại diện riêng biệt theo số thứ tự tài khoản */}
        <AvatarCircle
          size={38}
          text={account.avatarText}
          bg={themeColor.hex}
          textColor="#FFFFFF"
          className="font-bold text-[13px] shadow-xs shrink-0 ring-2 ring-white"
        />

        <div className="flex flex-col gap-0.5 overflow-hidden">
          <div className="flex items-center gap-2">
            <span className="text-[14px] font-semibold text-[#202733] truncate">
              {account.name}
            </span>

            {/* Trạng thái kết nối của tài khoản */}
            {account.isConnected ? (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-[#E9F5EF] text-[#25805C] text-[10.5px] font-medium rounded-full shrink-0">
                <CheckCircle2 className="w-3 h-3 text-[#25805C]" />
                Đã kết nối
              </span>
            ) : account.isAuthenticating ? (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-[#EFF6FF] text-[#0866FF] text-[10.5px] font-medium rounded-full shrink-0">
                <RefreshCw className="w-3 h-3 animate-spin text-[#0866FF]" />
                Đang chờ đăng nhập...
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-[#FFF3E8] text-[#D97706] text-[10.5px] font-medium rounded-full shrink-0">
                <AlertCircle className="w-3 h-3 text-[#D97706]" />
                Chưa kết nối
              </span>
            )}
          </div>

          <span className="text-[11.5px] text-[#6B7483] truncate">
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
                <RefreshCw className="w-3.5 h-3.5 animate-spin text-[#0866FF]" />
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
                className="p-2 rounded-[6px] text-[#6B7483] hover:text-[#DC2626] hover:bg-[#FEE2E2]/40 border border-[#E2E7EF] transition-all cursor-pointer"
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
                <LogIn className="w-3.5 h-3.5 text-[#0866FF]" />
              )
            }
            onClick={() => onOpen(account)}
          >
            {account.isConnected ? "Mở Messenger" : "Đăng nhập"}
          </CommonButton>
        )}

        {/* Nút Làm mới phiên */}
        {/* {account.isConnected && (
          <button
            type="button"
            title="Làm mới phiên tài khoản này"
            onClick={() => onRefresh(account)}
            disabled={isRefreshing}
            className="p-2 rounded-[6px] text-[#6B7483] hover:text-[#0866FF] hover:bg-white border border-transparent hover:border-[#E2E7EF] transition-all disabled:opacity-50 cursor-pointer"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
          </button>
        )} */}

        {/* Nút Gỡ bỏ tài khoản */}
        <button
          type="button"
          title={`Gỡ bỏ ${account.name}`}
          onClick={() => onRemove(account)}
          className="p-2 rounded-[6px] text-[#94A3B8] hover:text-[#DC2626] hover:bg-[#FEE2E2]/40 border border-transparent hover:border-[#FECACA] transition-all cursor-pointer"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
