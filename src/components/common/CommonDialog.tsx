import React, { useEffect, useCallback } from "react";
import { X, RefreshCw } from "lucide-react";
import { CommonButton, CommonButtonProps } from "./CommonButton";

export type DialogSize = "sm" | "md" | "lg" | "xl";
export type DialogFooterLayout = "split" | "end";

export interface DialogActionProps {
  label?: string;
  onClick?: (e?: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  variant?: CommonButtonProps["variant"];
  iconLeading?: React.ReactNode;
  iconTrailing?: React.ReactNode;
  disabled?: boolean;
  loading?: boolean;
  className?: string;
}

export interface CommonDialogProps {
  isOpen: boolean;
  onClose: () => void;
  title: React.ReactNode;
  titleIcon?: React.ReactNode;
  eyebrow?: string;
  content?: React.ReactNode;
  widget?: React.ReactNode;
  backButton?: boolean | DialogActionProps | React.ReactNode;
  rightButton?: DialogActionProps | React.ReactNode;
  footerLayout?: DialogFooterLayout;
  size?: DialogSize;
  showCloseButton?: boolean;
  closeOnBackdropClick?: boolean;
  closeOnEscape?: boolean;
  className?: string;
  bodyClassName?: string;
}

const DIALOG_SIZE_MAP: Record<DialogSize, string> = {
  sm: "w-dialog-sm max-w-[90vw]",
  md: "w-dialog-md max-w-[90vw]",
  lg: "w-dialog-lg max-w-[90vw]",
  xl: "w-dialog-xl max-w-[90vw]",
};

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

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (closeOnBackdropClick && e.target === e.currentTarget) {
        onClose();
      }
    },
    [closeOnBackdropClick, onClose],
  );

  if (!isOpen) return null;

  const renderAction = (
    action: boolean | DialogActionProps | React.ReactNode | undefined,
    defaultLabel: string,
    defaultVariant: CommonButtonProps["variant"],
    fallbackOnClick?: () => void,
  ): React.ReactNode => {
    if (!action) return null;
    if (React.isValidElement(action)) {
      return action;
    }
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

  const renderedBack = renderAction(backButton, "Cancel", "outline", onClose);
  const renderedRight = renderAction(rightButton, "Confirm", "primary");
  const hasFooter = Boolean(renderedBack || renderedRight);

  return (
    <div
      role="dialog"
      aria-modal="true"
      onClick={handleBackdropClick}
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-content-primary/45 backdrop-blur-xs animate-in fade-in duration-150 select-none"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className={`${DIALOG_SIZE_MAP[size]} bg-background-surface border border-border rounded-modal shadow-window flex flex-col max-h-[90vh] overflow-hidden animate-in zoom-in-95 duration-150 ${className}`}
      >
        <div className="px-6 pt-5 pb-4 border-b border-border flex items-start justify-between gap-4 shrink-0 bg-background-surface">
          <div className="flex flex-col gap-1 min-w-0">
            {eyebrow && (
              <span className="text-caption font-semibold text-content-secondary uppercase tracking-wider leading-none">
                {eyebrow}
              </span>
            )}
            <div className="flex items-center gap-2.5">
              {titleIcon && (
                <span className="shrink-0 flex items-center text-primary">
                  {titleIcon}
                </span>
              )}
              <h2 className="text-section font-semibold text-content-primary tracking-tight leading-snug m-0 truncate">
                {title}
              </h2>
            </div>
          </div>
          {showCloseButton && (
            <button
              type="button"
              onClick={onClose}
              title="Close (Esc)"
              aria-label="Close"
              className="w-7 h-7 -mr-1.5 -mt-1 rounded-control flex items-center justify-center text-content-secondary hover:text-content-primary hover:bg-background-app active:bg-border transition-colors cursor-pointer shrink-0"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        <div
          className={`px-6 py-5 overflow-y-auto flex-1 flex flex-col gap-4.5 custom-scrollbar ${bodyClassName}`}
        >
          {content && (
            <div className="text-body text-content-secondary leading-relaxed">
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
        {hasFooter && (
          <div
            className={`px-6 py-3.5 bg-background-subtle border-t border-border flex items-center gap-3 shrink-0 ${
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