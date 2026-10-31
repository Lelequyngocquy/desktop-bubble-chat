import React from "react";
import { HardDrive, Trash2, RefreshCw, Check } from "lucide-react";

interface DataCacheSectionProps {
  clearStatus: string | null;
  onOpenClearDialog: () => void;
}

export const DataCacheSection: React.FC<DataCacheSectionProps> = ({
  clearStatus,
  onOpenClearDialog,
}) => {
  return (
    <div className="bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
      <div className="flex items-center gap-2 pb-2 border-b border-[#E2E7EF]/60">
        <HardDrive className="w-4 h-4 text-[#0866FF]" />
        <h2 className="text-[15px] font-semibold text-[#1E293B] m-0">
          Dữ liệu & Bộ nhớ đệm
        </h2>
      </div>

      <div className="flex items-center justify-between gap-4 py-1">
        <div className="flex flex-col gap-0.5">
          <span className="text-[13px] font-medium text-[#1E293B]">
            Xóa bộ nhớ đệm WebView2
          </span>
          <span className="text-[11px] text-[#64748B]">
            Giải phóng dung lượng ổ đĩa và làm mới hoàn toàn session nếu gặp
            lỗi hiển thị
          </span>
        </div>
        <button
          type="button"
          onClick={onOpenClearDialog}
          disabled={clearStatus === "clearing"}
          className="px-3.5 py-1.5 border border-[#E2E7EF] rounded-lg text-[12px] font-medium text-[#DC2626] hover:bg-[#FEECEE] flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
        >
          {clearStatus === "clearing" ? (
            <>
              <RefreshCw className="w-3.5 h-3.5 animate-spin" />
              <span>Đang dọn dẹp...</span>
            </>
          ) : clearStatus === "success" ? (
            <>
              <Check className="w-3.5 h-3.5 text-green-600" />
              <span className="text-green-600">Đã xóa thành công!</span>
            </>
          ) : (
            <>
              <Trash2 className="w-3.5 h-3.5" />
              <span>Xóa bộ nhớ đệm</span>
            </>
          )}
        </button>
      </div>

      <div className="pt-2 border-t border-[#E2E7EF]/50 flex items-center justify-between text-[11px] text-[#94A3B8]">
        <span>DesktopBubbleChat v0.1.0 (Windows 11 / WebView2)</span>
        <span>Build Clean Architecture</span>
      </div>
    </div>
  );
};

export default DataCacheSection;
