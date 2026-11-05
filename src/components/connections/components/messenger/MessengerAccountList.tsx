import React, { useState } from "react";
import { Users, Trash2, Moon } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  MAX_MESSENGER_ACCOUNTS,
  MessengerAccount,
} from "../../../../types/account";
import { useMessengerAccounts } from "../../../../platforms/messenger";
import { getAccountThemeColor } from "../../../../constants/design";
import { CommonDialog } from "../../../common";
import { MessengerAccountRow } from "./MessengerAccountRow";
import { AddAccountButton } from "./AddAccountButton";

export const MessengerAccountList: React.FC = () => {
  const { t } = useTranslation("dashboard");
  const {
    accounts,
    refreshingId,
    openAccount,
    cancelAuth,
    refreshAccount,
    removeAccount,
    addAccount,
  } = useMessengerAccounts();
  const [accountToRemove, setAccountToRemove] = useState<MessengerAccount | null>(null);
  const [accountToWake, setAccountToWake] = useState<MessengerAccount | null>(null);
  const handleOpenAccount = (account: MessengerAccount) => {
    if (account.isConnected && !account.isBubbleVisible) {
      setAccountToWake(account);
      return;
    }
    openAccount(account);
  };

  return (
    <div className="flex flex-col gap-3">
      {accounts.length === 0 ? (
        <div className="p-6 rounded-card-sm bg-background-subtle border border-border flex flex-col items-center justify-center text-center gap-2.5">
          <div className="w-avatar-lg h-avatar-lg rounded-pill bg-background-surface border border-border flex items-center justify-center text-primary">
            <Users className="w-5 h-5" />
          </div>
          <div className="flex flex-col gap-0.5">
            <span className="text-nav font-semibold text-content-primary">
              {t("connections_page.empty_accounts_title")}
            </span>
            <span className="text-small text-content-secondary max-w-[340px]">
              {t("connections_page.empty_accounts_desc")}
            </span>
          </div>
        </div>
      ) : (
        <div className="flex flex-col gap-2.5">
          {accounts.map((account) => (
            <MessengerAccountRow
              key={account.id}
              account={account}
              onOpen={handleOpenAccount}
              onRefresh={refreshAccount}
              onRemove={(acc) => setAccountToRemove(acc)}
              onCancelAuth={cancelAuth}
              isRefreshing={refreshingId === account.id}
            />
          ))}
        </div>
      )}
      <AddAccountButton
        count={accounts.length}
        maxCount={MAX_MESSENGER_ACCOUNTS}
        onAdd={addAccount}
      />
      <CommonDialog
        isOpen={Boolean(accountToRemove)}
        onClose={() => setAccountToRemove(null)}
        eyebrow={t("connections_page.dialog_remove_eyebrow")}
        titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
        title={t("connections_page.dialog_remove_title", {
          name: accountToRemove?.name || "",
        })}
        content={t("connections_page.dialog_remove_content", {
          name: accountToRemove?.name || "",
        })}
        widget={
          accountToRemove && (
            <div className="p-3 bg-background-subtle rounded-panel border border-border flex flex-col gap-1.5 text-small text-content-secondary">
              <div className="flex items-center gap-2 font-medium text-content-primary">
                <span
                  className="w-2.5 h-2.5 rounded-pill"
                  style={{
                    backgroundColor: getAccountThemeColor(accountToRemove.index).hex,
                  }}
                />
                <span>
                  {t("connections_page.dialog_remove_widget_title", {
                    name: accountToRemove.name,
                    index: accountToRemove.index < 10
                      ? `0${accountToRemove.index}`
                      : accountToRemove.index,
                  })}
                </span>
              </div>
              <span className="text-caption">
                {accountToRemove.isConnected
                  ? t("connections_page.dialog_remove_connected_hint")
                  : t("connections_page.dialog_remove_disconnected_hint")}
              </span>
            </div>
          )
        }
        backButton={{
          label: t("connections_page.dialog_remove_cancel"),
          onClick: () => setAccountToRemove(null),
        }}
        rightButton={{
          label: t("connections_page.dialog_remove_confirm"),
          variant: "danger",
          onClick: async () => {
            if (accountToRemove) {
              const target = accountToRemove;
              setAccountToRemove(null);
              await removeAccount(target);
            }
          },
        }}
      />
      <CommonDialog
        isOpen={Boolean(accountToWake)}
        onClose={() => setAccountToWake(null)}
        eyebrow={t("connections_page.dialog_wake_eyebrow")}
        titleIcon={<Moon className="w-5 h-5 text-primary" />}
        title={t("connections_page.dialog_wake_title", {
          name: accountToWake?.name || "",
        })}
        content={t("connections_page.dialog_wake_content", {
          name: accountToWake?.name || "",
        })}
        widget={
          accountToWake && (
            <div className="p-3 bg-background-subtle rounded-panel border border-border flex flex-col gap-1.5 text-small text-content-secondary">
              <div className="flex items-center gap-2 font-medium text-content-primary">
                <span
                  className="w-2.5 h-2.5 rounded-pill"
                  style={{
                    backgroundColor: getAccountThemeColor(accountToWake.index).hex,
                  }}
                />
                <span>
                  {t("connections_page.dialog_remove_widget_title", {
                    name: accountToWake.name,
                    index: accountToWake.index < 10
                      ? `0${accountToWake.index}`
                      : accountToWake.index,
                  })}
                </span>
              </div>
              <span className="text-caption">
                {t("connections_page.dialog_wake_widget_hint")}
              </span>
            </div>
          )
        }
        backButton={{
          label: t("connections_page.dialog_wake_stay_hibernating"),
          onClick: () => setAccountToWake(null),
        }}
        rightButton={{
          label: t("connections_page.dialog_wake_confirm"),
          variant: "primary",
          onClick: async () => {
            if (accountToWake) {
              const target = accountToWake;
              setAccountToWake(null);
              await openAccount(target);
            }
          },
        }}
      />
    </div>
  );
};

export default MessengerAccountList;