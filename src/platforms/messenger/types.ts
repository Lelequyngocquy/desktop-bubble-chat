export * from "../../types/account";

export interface MessengerAuthStatus {
  connected: boolean;
  name?: string;
  accountIndex?: number;
  url?: string;
  title?: string;
}
