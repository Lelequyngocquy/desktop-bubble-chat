import React from "react";
import { Sparkles, MousePointer, Keyboard } from "lucide-react";
import { TipCard } from "./TipCard";
import { CommonLargeCard } from "../../common/index";

export const QuickTipsSection: React.FC = () => {
  return (
    <CommonLargeCard
      icon={<Sparkles className="w-4 h-4 text-[#0866FF]" />}
      title="Mẹo sử dụng nhanh"
    >
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <TipCard
          icon={<MousePointer className="w-5 h-5" />}
          title="Rê chuột 0.1s (Hover)"
          description="Rê chuột vào bong bóng bất kỳ trong 100ms để mở ngay khung chat mà không cần click. Di chuột ra ngoài 1 giây khung chat sẽ tự động đóng êm ái."
        />

        <TipCard
          icon={<Keyboard className="w-5 h-5" />}
          title="Phím tắt toàn cục"
          badge="Ctrl + Shift + M"
          description="Bấm tổ hợp phím này bất cứ lúc nào khi đang làm việc trên Windows để ẩn hoặc hiện toàn bộ bong bóng chat trên màn hình."
        />
      </div>
    </CommonLargeCard>
  );
};
