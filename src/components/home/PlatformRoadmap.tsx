import React from "react";
import { AvatarSquareRounded } from "../common";
import { UPCOMING_PLATFORMS } from "../../config";
import { PlatformIcon } from "../icons";

export const PlatformRoadmap: React.FC = () => {
  return (
    <div className="shrink-0 flex flex-col gap-3.5 select-none">
      {/* Section Heading (Figma Node #3:1336) */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1">
        <h2 className="text-[17px] font-semibold text-[#202733] m-0">
          Sắp có thêm nhiều kết nối
        </h2>
        <span className="text-[12px] text-[#6B7483]">
          Phiên bản đầu hiện chỉ hỗ trợ Messenger
        </span>
      </div>

      {/* Upcoming Platforms Cards (Figma Node #3:1339) */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3.5">
        {UPCOMING_PLATFORMS.map((p) => (
          <div
            key={p.id}
            className="h-[86px] p-4 bg-white/65 hover:bg-white/80 border border-[#E2E7EF] rounded-[8px] flex items-center gap-3 transition-colors shadow-2xs backdrop-blur-xs"
          >
            {/* Platform Icon container */}
            <AvatarSquareRounded
              size={38}
              radius={8}
              bg="#EEF2F9"
              icon={
                <PlatformIcon
                  platformId={p.id}
                  size={20}
                  className="text-[#202733]"
                />
              }
            />

            {/* Platform Information */}
            <div className="flex flex-col gap-1 flex-1 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-[14px] font-medium text-[#202733] truncate">
                  {p.name}
                </span>
                <span className="px-[7px] py-[2px] bg-[#EEF2F9] text-[#6B7483] text-[10px] rounded-[6px] shrink-0 font-normal">
                  {p.upcomingBadgeText || "Sắp ra mắt"}
                </span>
              </div>
              <span className="text-[11px] text-[#6B7483] truncate">
                {p.description}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
