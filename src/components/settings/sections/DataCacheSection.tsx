import React from "react";
import { HardDrive, Trash2, RefreshCw, Check } from "lucide-react";
import { CommonLargeCard } from "../../common";

interface DataCacheSectionProps {
  clearStatus: string | null;
  onOpenClearDialog: () => void;
}

export const DataCacheSection: React.FC<DataCacheSectionProps> = ({
  clearStatus,
  onOpenClearDialog,
}) => {
  return (
    <CommonLargeCard
      icon={<HardDrive className="w-4 h-4 text-primary" />}
      title="Dữ liệu & Bộ nhớ đệm"
    >
      <div className="flex items-center justify-between gap-4 py-1">
        <div className="flex flex-col gap-0.5">
          <span className="text-body font-medium text-content-primary">
            Xóa bộ nhớ đệm WebView2
          </span>
          <span className="text-caption text-content-secondary">
            Giải phóng dung lượng ổ đĩa và làm mới hoàn toàn session nếu gặp lỗi
            hiển thị
          </span>
        </div>
        <button
          type="button"
          onClick={onOpenClearDialog}
          disabled={clearStatus === "clearing"}
          className="px-3.5 py-1.5 border border-border rounded-panel text-small font-medium text-status-error hover:bg-status-errorBg flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
        >
          {clearStatus === "clearing" ? (
            <>
              <RefreshCw className="w-3.5 h-3.5 animate-spin" />
              <span>Đang dọn dẹp...</span>
            </>
          ) : clearStatus === "success" ? (
            <>
              <Check className="w-3.5 h-3.5 text-status-success" />
              <span className="text-status-success">Đã xóa thành công!</span>
            </>
          ) : (
            <>
              <Trash2 className="w-3.5 h-3.5" />
              <span>Xóa bộ nhớ đệm</span>
            </>
          )}
        </button>
      </div>
    </CommonLargeCard>
  );
};

export default DataCacheSection;
