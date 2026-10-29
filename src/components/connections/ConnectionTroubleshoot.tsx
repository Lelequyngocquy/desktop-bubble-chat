import React from "react";
import { HelpCircle, ArrowRight } from "lucide-react";
import { MOCK_TROUBLESHOOT_STEPS } from "../../mocks";

export const ConnectionTroubleshoot: React.FC = () => {
  return (
    <div className="shrink-0 bg-white border border-[#E2E7EF] rounded-[12px] shadow-card p-5 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <HelpCircle className="w-[17px] h-[17px] text-[#6B7483]" />
          <h3 className="text-[16px] font-semibold text-[#202733] m-0">
            Gặp vấn đề kết nối?
          </h3>
        </div>
        <button className="flex items-center gap-1 text-[12px] font-medium text-[#0759DB] hover:underline">
          <span>Trợ giúp & phản hồi</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        {MOCK_TROUBLESHOOT_STEPS.map((step) => (
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
    </div>
  );
};
