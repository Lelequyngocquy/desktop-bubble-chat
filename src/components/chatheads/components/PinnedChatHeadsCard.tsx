import React from "react";
import { Pin } from "lucide-react";
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
  return (
    <CommonLargeCard
      icon={<Pin className="w-4 h-4 text-[#0866FF] fill-current -rotate-45" />}
      title={`Cuộc trò chuyện đã ghim (${pinnedList.length}/6)`}
      subTitle="Luôn hiển thị đầu tiên"
    >
      {pinnedList.length === 0 ? (
        <div className="py-8 text-center text-[#64748B] text-[12px]">
          Chưa có cuộc trò chuyện nào được ghim. Nhấn vào biểu tượng ghim ở danh
          sách bên dưới để cố định tối đa 6 người liên lạc.
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
