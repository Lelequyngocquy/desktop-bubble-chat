import { ConnectedAccount, SystemUserProfile } from "../types/account";

export const MOCK_FALLBACK_USER: SystemUserProfile = {
  name: "User",
  avatarText: "U",
};

export const MOCK_DEFAULT_CONNECTED_ACCOUNT: ConnectedAccount = {
  name: "Messenger Account",
  avatarText: "FB",
  lastSynced: "Just synced",
};
