import React from "react";
import { Plus, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";

interface AddAccountButtonProps {
  count: number;
  maxCount: number;
  onAdd: () => void;
}

export const AddAccountButton: React.FC<AddAccountButtonProps> = ({
  count,
  maxCount,
  onAdd,
}) => {
  const { t } = useTranslation("dashboard");
  const isMaxReached = count >= maxCount;

  if (isMaxReached) {
    return (
      <div className="w-full py-3 px-4 rounded-card-sm bg-background-subtle border border-border flex items-center justify-center gap-2 text-small font-medium text-content-secondary select-none">
        <ShieldAlert className="w-4 h-4 text-status-warning" />
        <span>
          {t("connections_page.max_limit_reached", { max: maxCount, count })}
        </span>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={onAdd}
      className="w-full py-3 px-4 rounded-card-sm border-2 border-dashed border-border hover:border-primary bg-background-surface hover:bg-primary-subtle text-primary flex items-center justify-center gap-2 text-body font-semibold transition-all cursor-pointer shadow-xs active:scale-[0.99] group"
    >
      <div className="w-5 h-5 rounded-pill bg-primary-subtle group-hover:bg-primary flex items-center justify-center transition-colors">
        <Plus className="w-3.5 h-3.5 text-primary group-hover:text-content-inverse transition-colors" />
      </div>
      <span>{t("connections_page.add_account_btn")}</span>
      <span className="text-caption font-normal px-2 py-0.5 rounded-pill bg-primary-subtle text-primary">
        {count}/{maxCount}
      </span>
    </button>
  );
};

export default AddAccountButton;