import React, { useState } from "react";
import { Users, Trash2, Moon } from "lucide-react";
import {
  MAX_MESSENGER_ACCOUNTS,
  MessengerAccount,
} from "../../../../types/account";
import { useMessengerAccounts } from "../../../../platforms/messenger";
import { getAccountThemeColor } from "../../../../constants/design";
import { CommonDialog } from "../../../common";
import { MessengerAccountRow } from "./MessengerAccountRow";
import { AddAccountButton } from "./AddAccountButton";

/**
 * Presentational component rendering the Messenger accounts list.
 * Follows Single Responsibility Principle (SRP): Renders UI only,
 * delegating all orchestration to useMessengerAccounts hook.
 */
export const MessengerAccountList: React.FC = () => {
  const {
    accounts,
    refreshingId,
    openAccount,
    cancelAuth,
    refreshAccount,
    removeAccount,
    addAccount,
  } = useMessengerAccounts();

  const [accountToRemove, setAccountToRemove] =
    useState<MessengerAccount | null>(null);
  const [accountToWake, setAccountToWake] =
    useState<MessengerAccount | null>(null);

  const handleOpenAccount = (account: MessengerAccount) => {
    // Nếu tài khoản đã kết nối nhưng đang ngủ đông (bong bóng không hiển thị)
    if (account.isConnected && !account.isBubbleVisible) {
      setAccountToWake(account);
      return;
    }
    openAccount(account);
  };

  return (
    <div className="flex flex-col gap-3">
      {/* Empty State */}
      {accounts.length === 0 ? (
        <div className="p-6 rounded-card-sm bg-background-subtle border border-border flex flex-col items-center justify-center text-center gap-2.5">
          <div className="w-avatar-lg h-avatar-lg rounded-pill bg-primary-subtle flex items-center justify-center text-primary">
            <Users className="w-5 h-5" />
          </div>
          <div className="flex flex-col gap-0.5">
            <span className="text-nav font-semibold text-content-primary">
              Chưa có tài khoản Messenger nào
            </span>
            <span className="text-small text-content-secondary max-w-[340px]">
              Bấm dấu cộng bên dưới để thêm tài khoản đầu tiên (hỗ trợ tối đa 5
              tài khoản độc lập).
            </span>
          </div>
        </div>
      ) : (
        /* Accounts list */
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

      {/* Add Account Button */}
      <AddAccountButton
        count={accounts.length}
        maxCount={MAX_MESSENGER_ACCOUNTS}
        onAdd={addAccount}
      />

      {/* Dialog xác nhận gỡ bỏ tài khoản Messenger */}
      <CommonDialog
        isOpen={Boolean(accountToRemove)}
        onClose={() => setAccountToRemove(null)}
        eyebrow="QUẢN LÝ TÀI KHOẢN"
        titleIcon={<Trash2 className="w-5 h-5 text-status-error" />}
        title={`Gỡ bỏ ${accountToRemove?.name || "tài khoản"}`}
        content={`Bạn có chắc chắn muốn gỡ bỏ tài khoản ${accountToRemove?.name}? Thao tác này sẽ đóng phiên làm việc, xóa dữ liệu đệm cục bộ và tắt bong bóng nổi tương ứng.`}
        widget={
          accountToRemove && (
            <div className="p-3 bg-background-subtle rounded-panel border border-border flex flex-col gap-1.5 text-small text-content-secondary">
              <div className="flex items-center gap-2 font-medium text-content-primary">
                <span
                  className="w-2.5 h-2.5 rounded-pill"
                  style={{
                    backgroundColor: getAccountThemeColor(accountToRemove.index)
                      .hex,
                  }}
                />
                <span>
                  {accountToRemove.name} (Vị trí Messenger{" "}
                  {accountToRemove.index < 10
                    ? `0${accountToRemove.index}`
                    : accountToRemove.index}
                  )
                </span>
              </div>
              <span className="text-caption">
                {accountToRemove.isConnected
                  ? "Tài khoản hiện đang kết nối. Gỡ bỏ sẽ hủy phiên đăng nhập này."
                  : "Tài khoản chưa đăng nhập. Bạn có thể thêm lại bất cứ lúc nào."}
              </span>
            </div>
          )
        }
        backButton={{
          label: "Hủy bỏ",
          onClick: () => setAccountToRemove(null),
        }}
        rightButton={{
          label: "Xác nhận gỡ bỏ",
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

      {/* Dialog xác nhận cho phép hoạt động khi tài khoản đang ngủ đông */}
      <CommonDialog
        isOpen={Boolean(accountToWake)}
        onClose={() => setAccountToWake(null)}
        eyebrow="TRẠNG THÁI TÀI KHOẢN"
        titleIcon={<Moon className="w-5 h-5 text-primary" />}
        title={`Kích hoạt ${accountToWake?.name || "tài khoản"}?`}
        content={`Tài khoản ${accountToWake?.name} hiện đang ở trạng thái ngủ đông (0 MB RAM). Bạn có muốn cho phép tài khoản hoạt động để bật lại bong bóng nổi và mở khung chat Messenger không?`}
        widget={
          accountToWake && (
            <div className="p-3 bg-background-subtle rounded-panel border border-border flex flex-col gap-1.5 text-small text-content-secondary">
              <div className="flex items-center gap-2 font-medium text-content-primary">
                <span
                  className="w-2.5 h-2.5 rounded-pill"
                  style={{
                    backgroundColor: getAccountThemeColor(accountToWake.index)
                      .hex,
                  }}
                />
                <span>
                  {accountToWake.name} (Vị trí Messenger{" "}
                  {accountToWake.index < 10
                    ? `0${accountToWake.index}`
                    : accountToWake.index}
                  )
                </span>
              </div>
              <span className="text-caption">
                Khi cho phép hoạt động, bong bóng nổi sẽ xuất hiện trên màn hình và nạp phiên làm việc WebView2 để nhận tin nhắn.
              </span>
            </div>
          )
        }
        backButton={{
          label: "Tiếp tục ngủ đông",
          onClick: () => setAccountToWake(null),
        }}
        rightButton={{
          label: "Cho phép hoạt động",
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
