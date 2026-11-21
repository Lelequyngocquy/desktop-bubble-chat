export interface UnreadListenerContext {
  register: <T>(eventName: string, onPayload: (payload: T) => void) => Promise<boolean>;
  isDisposed: () => boolean;
  prevUnreadMap: Map<number, number>;
  isInitialGracePeriod: { current: boolean };
}
