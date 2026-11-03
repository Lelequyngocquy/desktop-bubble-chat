import React from "react";
import { Lightbulb } from "lucide-react";

export const HomeFooter: React.FC = () => {
  return (
    <footer className="shrink-0 pt-1 flex items-center justify-between text-caption text-content-secondary select-none flex-wrap gap-2">
      <div className="flex items-center gap-1.5">
        <Lightbulb className="w-[13px] h-[13px] text-content-secondary shrink-0" />
        <span>
          Mẹo: Ẩn chat head không xóa cuộc trò chuyện của bạn trên Messenger.
        </span>
      </div>
    </footer>
  );
};

export default HomeFooter;
