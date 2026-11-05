export interface TroubleshootStep {
  num: number;
  title: string;
  desc: string;
}

export const TROUBLESHOOT_STEPS: TroubleshootStep[] = [
  {
    num: 1,
    title: "Check Internet",
    desc: "Ensure your computer has a stable network connection.",
  },
  {
    num: 2,
    title: "Check Messenger login",
    desc: "Log in again if your authentication session on this device has expired.",
  },
  {
    num: 3,
    title: "Try reconnecting",
    desc: "Use the Reconnect action if unread messages are not synchronizing.",
  },
];