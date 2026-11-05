import React from "react";
import { Volume2, VolumeX } from "lucide-react";
import { useTranslation } from "react-i18next";
import { MessengerAccount } from "../../../../types/account";
import { AvatarCircle, SwitchButton } from "../../../common";
import { useMessengerStore } from "../../../../platforms/messenger";
import { Color, Dimen } from "../../../../constants/design";

interface ConnectedAccountRowProps {
  account: MessengerAccount;
  onOpenChat: (account: MessengerAccount) => void;
  onToggleBubble: (account: MessengerAccount) => void;
}

export const ConnectedAccountRow: React.FC<ConnectedAccountRowProps> = ({
  account,
  onToggleBubble,
}) => {
  const { t } = useTranslation("dashboard");
  const toggleAccountSound = useMessengerStore((s) => s.toggleAccountSound);
  const isSoundOn = account.soundEnabled ?? true;

  return (
    <div className="flex items-center justify-between p-3 bg-background-subtle hover:bg-background-workspace rounded-card-sm border border-border/80 transition-all gap-3 flex-wrap sm:flex-nowrap">
      <div className="flex items-center gap-3 min-w-0">
        <div className="relative shrink-0">
          <AvatarCircle
            size={Dimen.layout.avatarMd}
            text={account.avatarText}
            bg={account.color}
            textColor={Color.text.inverse}
            className="font-bold text-small shadow-xs ring-2 ring-white"
          />
          <span className="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-pill bg-status-success ring-2 ring-white" />
        </div>
        <div className="flex flex-col min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-body font-semibold text-content-primary truncate leading-tight">
              {account.name}
            </span>
            {account.unreadCount > 0 && (
              <span className="px-1.5 py-0.5 rounded-pill bg-status-error text-content-inverse text-badge font-bold shrink-0 leading-none">
                {account.unreadCount > 99 ? "99+" : account.unreadCount}
              </span>
            )}
          </div>
          <span className="text-caption text-content-secondary truncate leading-tight mt-0.5">
            {account.isBubbleVisible
              ? t("bubble_active")
              : t("bubble_hibernating")}{" "}
            • {account.lastSynced || t("synced_just_now")}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-2.5 shrink-0 self-end sm:self-center">
        <button
          type="button"
          onClick={() => toggleAccountSound(account.index)}
          title={isSoundOn ? t("sound_on_title") : t("sound_off_title")}
          className={`p-1.5 rounded-control border transition-all cursor-pointer ${
            isSoundOn
              ? "text-content-secondary hover:text-primary hover:bg-background-surface border-transparent hover:border-border"
              : "text-status-error bg-status-errorBg border-status-errorBorder"
          }`}
        >
          {isSoundOn ? (
            <Volume2 className="w-3.5 h-3.5" />
          ) : (
            <VolumeX className="w-3.5 h-3.5" />
          )}
        </button>
        <div className="flex items-center gap-1.5 pl-1.5 border-l border-border">
          <span className="text-caption text-content-secondary hidden md:inline">
            {t("bubble_label")}
          </span>
          <SwitchButton
            checked={account.isBubbleVisible}
            onChange={() => onToggleBubble(account)}
            aria-label={t("bubble_aria_label", { name: account.name })}
          />
        </div>
      </div>
    </div>
  );
};

export default ConnectedAccountRow;