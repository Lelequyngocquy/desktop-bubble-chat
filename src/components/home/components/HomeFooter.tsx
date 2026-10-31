import React from "react";
import { Lightbulb, ArrowUpRight } from "lucide-react";

export const HomeFooter: React.FC = () => {
  return (
    <footer className="shrink-0 pt-1 flex items-center justify-between text-[11px] text-[#6B7483] select-none flex-wrap gap-2">
      <div className="flex items-center gap-1.5">
        <Lightbulb className="w-[13px] h-[13px] text-[#6B7483] shrink-0" />
        <span>Mẹo: Ẩn chat head không xóa cuộc trò chuyện của bạn trên Messenger.</span>
      </div>

      <a
        href="https://github.com"
        target="_blank"
        rel="noreferrer"
        className="flex items-center gap-1 text-[#0759DB] hover:text-[#0866FF] transition-colors"
      >
        <span>Gửi góp ý</span>
        <ArrowUpRight className="w-[13px] h-[13px]" />
      </a>
    </footer>
  );
};
