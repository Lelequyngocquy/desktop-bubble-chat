import React from "react";
import { useTranslation } from "react-i18next";
import { Scaling, Info } from "lucide-react";
import { Color } from "../../../constants/design";
import { useMessengerStore } from "../../../platforms/messenger/stores/messengerStore";
import { useSettingsStore, DEFAULT_CHAT_WIDTH, DEFAULT_CHAT_HEIGHT } from "../../../stores/settingsStore";
import { CommonLargeCard, CommonBadge, CommonHintLine, AvatarCircle } from "../../common";

export const ChatWindowCustomizationSection: React.FC = () => {
  const { t } = useTranslation(["settings"]);
  const accounts = useMessengerStore((s) => s.accounts);
  const chatWindowSettings = useSettingsStore((s) => s.chatWindowSettings);
  const setChatWindowMode = useSettingsStore((s) => s.setChatWindowMode);
  const connectedAccounts = accounts.filter((acc) => acc.isConnected);
  return (
    <CommonLargeCard
      icon={<Scaling className="w-4 h-4 text-primary" />}
      title={t("settings:chat_window.title")}
    >
      <div className="flex flex-col gap-3">
        <p className="text-small text-content-secondary m-0">
          {t("settings:chat_window.subtitle")}
        </p>
        {connectedAccounts.length === 0 ? (
          <CommonHintLine
            icon={<Info className="w-4 h-4 text-content-secondary" />}
            text={t("settings:chat_window.no_accounts")}
            color="text-content-secondary"
            className="p-3"
          />
        ) : (
          <div className="flex flex-col gap-2.5 pt-1">
            {connectedAccounts.map((acc) => {
              const setting = chatWindowSettings.find((s) => s.accountIndex === acc.index);
              const isCustom = setting?.mode === "remember_custom";
              const width = isCustom ? (setting?.customWidth || DEFAULT_CHAT_WIDTH) : DEFAULT_CHAT_WIDTH;
              const height = isCustom ? (setting?.customHeight || DEFAULT_CHAT_HEIGHT) : DEFAULT_CHAT_HEIGHT;
              return (
                <div
                  key={acc.id}
                  className="flex flex-col sm:flex-row sm:items-center justify-between p-3.5 rounded-control border border-border bg-background-app/40 hover:bg-background-app/70 gap-3 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <AvatarCircle
                      text={acc.avatarText}
                      bg={acc.color}
                      textColor={Color.text.inverse}
                      size={36}
                      className="font-bold text-white shadow-xs"
                    />
                    <div className="flex flex-col">
                      <div className="flex items-center gap-2">
                        <span className="text-body font-semibold text-content-primary leading-tight">
                          {acc.name}
                        </span>
                        <span className="text-caption px-1.5 py-0.5 rounded bg-background-subtle border border-border text-content-muted">
                          #{acc.index.toString().padStart(2, "0")}
                        </span>
                      </div>
                      <div className="flex items-center gap-1.5 mt-1">
                        <CommonBadge
                          status={
                            isCustom
                              ? t("settings:chat_window.badge_custom", { width, height })
                              : t("settings:chat_window.badge_default", { width: DEFAULT_CHAT_WIDTH, height: DEFAULT_CHAT_HEIGHT })
                          }
                          color="primary"
                        />
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 self-end sm:self-auto">
                    <div className="flex items-center p-0.5 bg-background-subtle border border-border rounded-pill">
                      <button
                        type="button"
                        onClick={() => setChatWindowMode(acc.index, acc.id, "default")}
                        className={`px-3 py-1 rounded-pill text-caption font-medium transition-all cursor-pointer ${
                          !isCustom
                            ? "bg-primary text-white shadow-xs font-semibold"
                            : "text-content-secondary hover:text-content-primary"
                        }`}
                      >
                        {t("settings:chat_window.mode_default")}
                      </button>
                      <button
                        type="button"
                        onClick={() => setChatWindowMode(acc.index, acc.id, "remember_custom")}
                        className={`px-3 py-1 rounded-pill text-caption font-medium transition-all cursor-pointer ${
                          isCustom
                            ? "bg-primary text-white shadow-xs font-semibold"
                            : "text-content-secondary hover:text-content-primary"
                        }`}
                      >
                        {t("settings:chat_window.mode_custom")}
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </CommonLargeCard>
  );
};

export default ChatWindowCustomizationSection;
