import React from "react";
import { Pin } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Conversation } from "../../../types/conversation";
import { PinnedChatItem } from "./PinnedChatItem";
import { CommonLargeCard } from "../../common/index";

interface PinnedChatHeadsCardProps {
  pinnedList: Conversation[];
  onOpenThread: (item: Conversation) => void;
  onUnpin: (id: string) => void;
}

export const PinnedChatHeadsCard: React.FC<PinnedChatHeadsCardProps> = ({
  pinnedList,
  onOpenThread,
  onUnpin,
}) => {
  const { t } = useTranslation("dashboard");

  return (
    <CommonLargeCard
      icon={<Pin className="w-4 h-4 text-primary fill-current -rotate-45" />}
      title={t("chatheads_page.pinned_title", { count: pinnedList.length })}
      subTitle={t("chatheads_page.pinned_subtitle")}
    >
      {pinnedList.length === 0 ? (
        <div className="py-8 text-center text-content-secondary text-small">
          {t("chatheads_page.pinned_empty")}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {pinnedList.map((item) => (
            <PinnedChatItem
              key={item.id}
              item={item}
              onOpenThread={onOpenThread}
              onUnpin={onUnpin}
            />
          ))}
        </div>
      )}
    </CommonLargeCard>
  );
};

export default PinnedChatHeadsCard;