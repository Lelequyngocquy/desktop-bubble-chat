import React from "react";
import { HelpCircle, ArrowRight } from "lucide-react";
import { TROUBLESHOOT_STEPS } from "../../../../config";
import { CommonLargeCard } from "../../../common/index";
import { useNavigationStore } from "../../../../stores/navigationStore";

export const MessengerTroubleshoot: React.FC = () => {
  const setActiveTab = useNavigationStore((s) => s.setActiveTab);

  return (
    <CommonLargeCard
      icon={<HelpCircle className="w-4 h-4 text-[#0866FF]" />}
      title="Gặp vấn đề kết nối Messenger?"
      linkTitle={
        <button
          type="button"
          onClick={() => setActiveTab("help")}
          className="flex items-center gap-1 text-[12px] font-medium text-[#0759DB] hover:underline cursor-pointer"
        >
          <span>Trợ giúp & phản hồi</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      }
      className="shrink-0"
    >
      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        {TROUBLESHOOT_STEPS.map((step) => (
          <div key={step.num} className="flex items-start gap-2.5">
            <div className="w-5 h-5 rounded-full bg-[#EEF2F9] text-[#6B7483] font-semibold text-[11px] flex items-center justify-center shrink-0 mt-0.5">
              {step.num}
            </div>
            <div className="flex flex-col">
              <span className="text-[13px] font-medium text-[#202733]">
                {step.title}
              </span>
              <span className="text-[11px] text-[#6B7483] leading-relaxed">
                {step.desc}
              </span>
            </div>
          </div>
        ))}
      </div>
    </CommonLargeCard>
  );
};
