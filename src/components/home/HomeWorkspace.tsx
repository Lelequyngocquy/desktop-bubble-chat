import React from "react";
import { WelcomeHeader } from "./WelcomeHeader";
import { MessengerConnectionCard } from "./MessengerConnectionCard";
import { ActiveChatHeads } from "./ActiveChatHeads";
import { QuickSettings } from "./QuickSettings";
import { PlatformRoadmap } from "./PlatformRoadmap";
import { HomeFooter } from "./HomeFooter";

export const HomeWorkspace: React.FC = () => {
  return (
    <div className="flex-1 bg-[#F4F7FC] overflow-y-auto p-[28px_32px_24px] flex flex-col gap-6 custom-scrollbar">
      {/* 1. Welcome & State Header (Figma Node #3:1199) */}
      <WelcomeHeader />

      {/* 2. Messenger Hero Connection Card (Figma Node #3:1207) */}
      <MessengerConnectionCard />

      {/* 3. Middle 2-Column Controls Grid (Figma Node #3:1248) */}
      <div className="flex flex-col lg:flex-row items-stretch gap-6">
        {/* Left: Pinned Active Chat Heads (Figma Node #3:1249) */}
        <ActiveChatHeads />

        {/* Right: Quick Settings Switch Box (Figma Node #3:1299) */}
        <QuickSettings />
      </div>

      {/* 4. Upcoming Platform Roadmap (Figma Node #3:1335) */}
      <PlatformRoadmap />

      {/* 5. Footer Tips & Feedback (Figma Node #3:1367) */}
      <HomeFooter />
    </div>
  );
};
