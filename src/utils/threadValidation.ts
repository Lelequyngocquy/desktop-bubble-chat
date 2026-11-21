const VALID_THREAD_ID_REGEX = /^[a-zA-Z0-9._-]+$/;

/**
 * Validates and trims a Messenger threadId, rejecting synthetic fallback IDs and unsafe characters.
 */
export function normalizeThreadId(threadId?: string | null): string | undefined {
  if (typeof threadId !== "string") return undefined;
  const trimmed = threadId.trim();
  if (!trimmed || trimmed.startsWith("sender-") || trimmed.startsWith("account-msg-")) return undefined;
  if (!VALID_THREAD_ID_REGEX.test(trimmed)) return undefined;
  return trimmed;
}

export function isValidThreadId(threadId?: string | null): threadId is string {
  return normalizeThreadId(threadId) !== undefined;
}
