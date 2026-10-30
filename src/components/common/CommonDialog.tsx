import React, { useEffect, useCallback } from "react";
import { X, RefreshCw } from "lucide-react";
import { CommonButton, CommonButtonProps } from "./CommonButton";

export type DialogSize = "sm" | "md" | "lg" | "xl";
export type DialogFooterLayout = "split" | "end";

export interface DialogActionProps {
  /** Nhãn chữ trên nút */
  label?: string;
  /** Callback khi click vào nút */
  onClick?: (e?: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  /** Kiểu biến thể màu sắc của nút */
  variant?: CommonButtonProps["variant"];
  /** Icon hiển thị phía trước chữ */
  iconLeading?: React.ReactNode;
  /** Icon hiển thị phía sau chữ */
  iconTrailing?: React.ReactNode;
  /** Vô hiệu hóa nút */
  disabled?: boolean;
  /** Trạng thái đang tải / xử lý (tự động xoay icon và disable nút) */
  loading?: boolean;
  /** Class CSS tùy biến thêm */
  className?: string;
}

export interface CommonDialogProps {
  /** Trạng thái mở/đóng của popup dialog */
  isOpen: boolean;
  /** Callback kích hoạt khi yêu cầu đóng dialog (bấm nút X, bấm ra ngoài backdrop, hoặc phím Escape) */
  onClose: () => void;

  /** Tiêu đề chính của Dialog */
  title: React.ReactNode;
  /** Icon bổ trợ hiển thị trước Title (tùy chọn) */
  titleIcon?: React.ReactNode;
  /** Nhãn phân loại nhỏ nằm phía trên Title (tùy chọn) */
  eyebrow?: string;

  /** Nội dung mô tả / văn bản hướng dẫn chính của Dialog */
  content?: React.ReactNode;

  /** Widget tùy biến chèn vào thân Dialog (Form, Input, Preview, Custom Component...) */
  widget?: React.ReactNode;

  /** Nút quay lại / hủy bỏ ở footer (tùy chọn: true để dùng mặc định, hoặc truyền cấu hình / ReactNode) */
  backButton?: boolean | DialogActionProps | React.ReactNode;

  /** Nút hành động chính bên phải footer (tùy chọn: truyền cấu hình hoặc ReactNode) */
  rightButton?: DialogActionProps | React.ReactNode;

  /** Cách bố trí các nút ở Footer: "split" (BackButton bên trái, RightButton bên phải) hoặc "end" (cùng dồn về bên phải) */
  footerLayout?: DialogFooterLayout;

  /** Kích thước chiều ngang của dialog (mặc định "md" - 460px) */
  size?: DialogSize;

  /** Hiển thị nút đóng (X) ở góc trên bên phải (mặc định: true) */
  showCloseButton?: boolean;

  /** Cho phép đóng khi click ra ngoài vùng backdrop mờ (mặc định: true) */
  closeOnBackdropClick?: boolean;

  /** Cho phép đóng khi nhấn phím Escape (mặc định: true) */
  closeOnEscape?: boolean;

  /** Tùy biến class cho khung Card Modal ngoài cùng */
  className?: string;
  /** Tùy biến class cho vùng chứa thân nội dung (Body) */
  bodyClassName?: string;
}

const DIALOG_SIZE_MAP: Record<DialogSize, string> = {
  sm: "max-w-[380px]",
  md: "max-w-[460px]",
  lg: "max-w-[560px]",
  xl: "max-w-[680px]",
};

/**
 * CommonDialog - Component Popup Modal chuẩn của hệ thống DesktopBubbleChat.
 * Tuân thủ nghiêm ngặt design tokens, typography và bảng màu chuẩn của ứng dụng.
 */
export const CommonDialog: React.FC<CommonDialogProps> = ({
  isOpen,
  onClose,
  title,
  titleIcon,
  eyebrow,
  content,
  widget,
  backButton,
  rightButton,
  footerLayout = "split",
  size = "md",
  showCloseButton = true,
  closeOnBackdropClick = true,
  closeOnEscape = true,
  className = "",
  bodyClassName = "",
}) => {
  // Lắng nghe phím Escape để đóng popup
  useEffect(() => {
    if (!isOpen || !closeOnEscape) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, closeOnEscape, onClose]);

  // Xử lý khi click vào backdrop mờ bên ngoài
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (closeOnBackdropClick && e.target === e.currentTarget) {
        onClose();
      }
    },
    [closeOnBackdropClick, onClose],
  );

  // Không hiển thị gì nếu dialog đang đóng
  if (!isOpen) return null;

  // Helper render các nút hành động trong footer
  const renderAction = (
    action: boolean | DialogActionProps | React.ReactNode | undefined,
    defaultLabel: string,
    defaultVariant: CommonButtonProps["variant"],
    fallbackOnClick?: () => void,
  ): React.ReactNode => {
    if (!action) return null;

    // Trường hợp truyền thẳng custom React element (ví dụ <button> hoặc <CommonButton>)
    if (React.isValidElement(action)) {
      return action;
    }

    // Trường hợp truyền boolean true -> dùng cấu hình mặc định
    const config: DialogActionProps =
      typeof action === "object" ? (action as DialogActionProps) : {};

    const isSpinning = Boolean(config.loading);
    const leadingIcon = isSpinning ? (
      <RefreshCw className="w-3.5 h-3.5 animate-spin" />
    ) : (
      config.iconLeading
    );

    return (
      <CommonButton
        variant={config.variant || defaultVariant}
        size="md"
        iconLeading={leadingIcon}
        iconTrailing={config.iconTrailing}
        disabled={config.disabled || isSpinning}
        className={config.className}
        onClick={config.onClick || fallbackOnClick}
      >
        {config.label || defaultLabel}
      </CommonButton>
    );
  };

  const renderedBack = renderAction(backButton, "Quay lại", "outline", onClose);
  const renderedRight = renderAction(rightButton, "Xác nhận", "primary");
  const hasFooter = Boolean(renderedBack || renderedRight);

  return (
    <div
      role="dialog"
      aria-modal="true"
      onClick={handleBackdropClick}
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-[#202733]/45 backdrop-blur-xs animate-in fade-in duration-150 select-none"
    >
      {/* Modal Dialog Card Container */}
      <div
        onClick={(e) => e.stopPropagation()}
        className={`w-full ${DIALOG_SIZE_MAP[size]} bg-white border border-[#E2E7EF] rounded-[12px] shadow-window flex flex-col max-h-[90vh] overflow-hidden animate-in zoom-in-95 duration-150 ${className}`}
      >
        {/* 1. Header */}
        <div className="px-6 pt-5 pb-4 border-b border-[#E2E7EF]/70 flex items-start justify-between gap-4 shrink-0 bg-white">
          <div className="flex flex-col gap-1 min-w-0">
            {eyebrow && (
              <span className="text-[11px] font-semibold text-[#6B7483] uppercase tracking-wider leading-none">
                {eyebrow}
              </span>
            )}
            <div className="flex items-center gap-2.5">
              {titleIcon && (
                <span className="shrink-0 flex items-center text-[#0866FF]">
                  {titleIcon}
                </span>
              )}
              <h2 className="text-[17px] font-semibold text-[#202733] tracking-tight leading-snug m-0 truncate">
                {title}
              </h2>
            </div>
          </div>

          {showCloseButton && (
            <button
              type="button"
              onClick={onClose}
              title="Đóng (Esc)"
              aria-label="Đóng"
              className="w-7 h-7 -mr-1.5 -mt-1 rounded-[6px] flex items-center justify-center text-[#6B7483] hover:text-[#202733] hover:bg-[#EEF2F9] active:bg-[#E2E7EF] transition-colors cursor-pointer shrink-0"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>

        {/* 2. Body (Content & Widget) */}
        <div
          className={`px-6 py-5 overflow-y-auto flex-1 flex flex-col gap-4.5 custom-scrollbar ${bodyClassName}`}
        >
          {content && (
            <div className="text-[13px] text-[#6B7483] leading-relaxed">
              {typeof content === "string" ? (
                <p className="m-0">{content}</p>
              ) : (
                content
              )}
            </div>
          )}

          {widget && (
            <div className="w-full mt-3 shrink-0 flex flex-col">{widget}</div>
          )}
        </div>

        {/* 3. Footer (BackButton & RightButton) */}
        {hasFooter && (
          <div
            className={`px-6 py-3.5 bg-[#F8FAFD] border-t border-[#E2E7EF] flex items-center gap-3 shrink-0 ${
              footerLayout === "end" ? "justify-end" : "justify-between"
            }`}
          >
            {footerLayout === "split" ? (
              <>
                <div className="flex items-center">{renderedBack}</div>
                <div className="flex items-center gap-2.5">{renderedRight}</div>
              </>
            ) : (
              <div className="flex items-center gap-2.5">
                {renderedBack}
                {renderedRight}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default CommonDialog;
