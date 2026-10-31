import React, { useState } from "react";
import { Users, Info, Trash2 } from "lucide-react";
import { MAX_MESSENGER_ACCOUNTS, MessengerAccount } from "../../../../types/account";
import { useMessengerAccounts } from "../../../../hooks/useMessengerAccounts";
import { getAccountThemeColor } from "../../../../constants/accountColors";
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

  const [accountToRemove, setAccountToRemove] = useState<MessengerAccount | null>(null);

  return (
    <div className="flex flex-col gap-3">
      {/* Empty State */}
      {accounts.length === 0 ? (
        <div className="p-6 rounded-[10px] bg-[#F8FAFD] border border-[#E2E7EF] flex flex-col items-center justify-center text-center gap-2.5">
          <div className="w-10 h-10 rounded-full bg-[#EAF2FF] flex items-center justify-center text-[#0866FF]">
            <Users className="w-5 h-5" />
          </div>
          <div className="flex flex-col gap-0.5">
            <span className="text-[14px] font-semibold text-[#202733]">
              Chưa có tài khoản Messenger nào
            </span>
            <span className="text-[12px] text-[#6B7483] max-w-[340px]">
              Bấm dấu cộng bên dưới để thêm tài khoản đầu tiên (hỗ trợ tối đa 5 tài khoản độc lập).
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
              onOpen={openAccount}
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

      {/* Context info banner */}
      <div className="flex items-start gap-2 pt-1 text-[11.5px] text-[#6B7483]">
        <Info className="w-3.5 h-3.5 text-[#0866FF] shrink-0 mt-0.5" />
        <span>
          Mỗi tài khoản được cấp một phân vùng lưu trữ (User Data Directory) riêng biệt, đảm bảo các phiên đăng nhập không bao giờ bị trùng hoặc đá nhau.
        </span>
      </div>

      {/* Dialog xác nhận gỡ bỏ tài khoản Messenger */}
      <CommonDialog
        isOpen={Boolean(accountToRemove)}
        onClose={() => setAccountToRemove(null)}
        eyebrow="QUẢN LÝ TÀI KHOẢN"
        titleIcon={<Trash2 className="w-5 h-5 text-[#DC2626]" />}
        title={`Gỡ bỏ ${accountToRemove?.name || "tài khoản"}`}
        content={`Bạn có chắc chắn muốn gỡ bỏ tài khoản ${accountToRemove?.name}? Thao tác này sẽ đóng phiên làm việc, xóa dữ liệu đệm cục bộ và tắt bong bóng nổi tương ứng.`}
        widget={
          accountToRemove && (
            <div className="p-3 bg-[#F8FAFD] rounded-[8px] border border-[#E2E7EF] flex flex-col gap-1.5 text-[12px] text-[#6B7483]">
              <div className="flex items-center gap-2 font-medium text-[#202733]">
                <span
                  className="w-2.5 h-2.5 rounded-full"
                  style={{
                    backgroundColor: getAccountThemeColor(accountToRemove.index).hex,
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
              <span className="text-[11.5px]">
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
    </div>
  );
};
