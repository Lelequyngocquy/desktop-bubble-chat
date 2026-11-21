import { listen, type EventCallback, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Registers an async Tauri event listener and immediately unsubscribes if disposed before resolving.
 */
export async function safeListen<T>(
  eventName: string,
  handler: EventCallback<T>,
  isDisposed: () => boolean,
  onSubscribed?: (unlisten: UnlistenFn) => void,
): Promise<UnlistenFn | undefined> {
  const unlisten = await listen<T>(eventName, handler);
  if (isDisposed()) {
    unlisten();
    return undefined;
  }
  onSubscribed?.(unlisten);
  return unlisten;
}

/**
 * Wraps an async subscription promise and cleans it up immediately if already disposed.
 */
export async function safeSubscribe(
  subscribePromise: Promise<(() => void) | undefined>,
  isDisposed: () => boolean,
  onSubscribed?: (unsub: () => void) => void,
): Promise<(() => void) | undefined> {
  const unsub = await subscribePromise;
  if (!unsub) return undefined;
  if (isDisposed()) {
    unsub();
    return undefined;
  }
  onSubscribed?.(unsub);
  return unsub;
}
