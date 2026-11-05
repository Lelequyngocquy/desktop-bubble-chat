import React from "react";
import { useTranslation } from "react-i18next";
import { AvatarSquareRounded, CommonBadge } from "../../common";
import { UPCOMING_PLATFORMS } from "../../../config";
import { PlatformIcon } from "../../icons";

export const PlatformRoadmap: React.FC = () => {
  const { t } = useTranslation("dashboard");

  return (
    <div className="shrink-0 flex flex-col gap-3.5 select-none">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1">
        <h2 className="text-section font-semibold text-content-primary m-0">
          {t("roadmap.title")}
        </h2>
        <span className="text-small text-content-secondary">
          {t("roadmap.subtitle")}
        </span>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3.5">
        {UPCOMING_PLATFORMS.map((p) => (
          <div
            key={p.id}
            className="h-[86px] p-4 bg-background-surface/65 hover:bg-background-surface/80 border border-border rounded-panel flex items-center gap-3 transition-colors shadow-2xs backdrop-blur-xs"
          >
            <AvatarSquareRounded
              size={38}
              bg="bg-background-subtle border border-border"
              icon={
                <PlatformIcon
                  platformId={p.id}
                  size={20}
                  className="text-content-primary"
                />
              }
            />
            <div className="flex flex-col gap-1 flex-1 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-nav font-medium text-content-primary truncate">
                  {p.name}
                </span>
                <CommonBadge
                  color="secondary"
                  status={p.upcomingBadgeText || t("roadmap.upcoming_badge")}
                />
              </div>
              <span className="text-caption text-content-secondary truncate">
                {p.description}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PlatformRoadmap;