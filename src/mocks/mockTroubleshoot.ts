export interface TroubleshootStep {
  num: number;
  title: string;
  desc: string;
}

export const MOCK_TROUBLESHOOT_STEPS: TroubleshootStep[] = [
  {
    num: 1,
    title: "Check Internet",
    desc: "Ensure your computer has a stable network connection.",
  },
  {
    num: 2,
    title: "Open Messenger to verify",
    desc: "Log in again if the authentication session has expired.",
  },
  {
    num: 3,
    title: "Try reconnecting",
    desc: "Use reconnect if message data is not updating properly.",
  },
];
