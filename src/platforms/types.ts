export {
  PlatformId,
  PlatformStatus,
  type PlatformConfig,
} from "../types/platform";

export interface IPlatformAdapter {
  id: string;
  name: string;
  openChat: (accountIndex: number) => Promise<void>;
  openAuth: (accountIndex: number) => Promise<void>;
  disconnect: (accountIndex: number) => Promise<void>;
}
