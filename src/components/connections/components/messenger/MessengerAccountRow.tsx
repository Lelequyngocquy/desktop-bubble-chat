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
import { useTranslation } from "react-i18next";
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

export const MessengerAccountRow: React.FC<MessengerAccountRowProps> = ({
  account,
  onOpen,
  onRemove,
  onCancelAuth,
}) => {
  const { t } = useTranslation("dashboard");
  const themeColor = getAccountThemeColor(account.index);

  return (
    <div className="flex items-center justify-between p-3.5 bg-background-subtle hover:bg-background-workspace rounded-card-sm border border-border transition-all gap-4 flex-wrap sm:flex-nowrap">
      <div className="flex items-center gap-3 min-w-0">
        <AvatarCircle
          size={Dimen.layout.avatarMd}
          text={account.avatarText}
          bg={themeColor.hex}
          textColor={Color.text.inverse}
          className="font-bold text-body shadow-xs shrink-0 ring-2 ring-background-subtle"
        />
        <div className="flex flex-col gap-0.5 overflow-hidden">
          <div className="flex items-center gap-2">
            <span className="text-nav font-semibold text-content-primary truncate">
              {account.name}
            </span>
            {account.isConnected ? (
              <div className="flex items-center gap-1.5 shrink-0">
                <CommonBadge
                  status={t("connections_page.status_connected")}
                  icon={<CheckCircle2 />}
                  color="success"
                />
                {account.isBubbleVisible ? (
                  <CommonBadge
                    status={t("connections_page.status_active")}
                    icon={<Activity />}
                    color="primary"
                  />
                ) : (
                  <CommonBadge
                    status={t("connections_page.status_hibernating")}
                    icon={<Moon />}
                    color="secondary"
                    title={t("connections_page.status_hibernating_tooltip")}
                  />
                )}
              </div>
            ) : account.isAuthenticating ? (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-primary-subtle text-primary text-caption font-medium rounded-pill shrink-0">
                <RefreshCw className="w-3 h-3 animate-spin text-primary" />
                {t("connections_page.status_authenticating")}
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-status-warningBg text-status-warning text-caption font-medium rounded-pill shrink-0">
                <AlertCircle className="w-3 h-3 text-status-warning" />
                {t("connections_page.status_disconnected")}
              </span>
            )}
          </div>
          <span className="text-caption text-content-secondary truncate">
            {account.isConnected
              ? t("connections_page.session_active_hint", {
                  synced: account.lastSynced || t("synced_just_now"),
                })
              : account.isAuthenticating
                ? t("connections_page.auth_opening_hint")
                : t("connections_page.login_prompt_hint")}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
        {account.isAuthenticating ? (
          <div className="flex items-center gap-1.5">
            <CommonButton
              variant="secondary"
              size="sm"
              iconTrailing={
                <RefreshCw className="w-3.5 h-3.5 animate-spin text-primary" />
              }
              onClick={() => onOpen(account)}
              title={t("connections_page.view_window_tooltip")}
            >
              {t("connections_page.view_window_btn")}
            </CommonButton>
            {onCancelAuth && (
              <button
                type="button"
                title={t("connections_page.cancel_auth_tooltip")}
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
            {account.isConnected
              ? t("connections_page.open_messenger_btn")
              : t("connections_page.login_btn")}
          </CommonButton>
        )}
        <button
          type="button"
          title={t("connections_page.remove_account_tooltip", {
            name: account.name,
          })}
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