import { beforeEach, describe, expect, it } from "vitest";
import {
  formatMessageTime,
  formatSenderName,
  useConversationStore,
} from "../conversationStore";

describe("conversation formatters", () => {
  it("falls back safely on missing or NaN timestamps", () => {
    expect(formatMessageTime(undefined)).toBe("Just now");
    expect(formatMessageTime(Number.NaN)).toBe("Just now");
  });

  it("formats recent timestamps and normalizes sender labels", () => {
    expect(formatMessageTime(Date.now())).toBe("Just now");
    expect(formatMessageTime(Date.now() - 5 * 60000)).toBe("5m ago");
    expect(formatSenderName("M02")).toBe("New message for Messenger 02");
    expect(formatSenderName("", 3)).toBe("New message for Messenger 03");
    expect(formatSenderName("Alice Nguyen", 1)).toBe("Alice Nguyen");
  });
});

describe("useConversationStore", () => {
  beforeEach(() => {
    useConversationStore.setState({ conversations: [] });
  });

  it("upserts messages cleanly, rejects synthetic thread IDs, and increments unread counts", () => {
    const store = useConversationStore.getState();
    store.upsertConversationFromMessage({
      accountIndex: 1,
      senderName: "Alice",
      body: "Hello",
      threadId: "123456",
      timestamp: Date.now(),
    });
    store.upsertConversationFromMessage({
      accountIndex: 1,
      senderName: "Alice",
      body: "Follow up",
      threadId: "sender-1-alice",
      timestamp: Date.now() + 1000,
    });
    const list = useConversationStore.getState().conversations;
    expect(list).toHaveLength(1);
    expect(list[0].unreadCount).toBe(2);
    expect(list[0].lastMessage).toBe("Follow up");
    expect(list[0].threadId).toBe("123456");
  });

  it("enforces max 6 pinned conversations and scopes clear by accountIndex", () => {
    const store = useConversationStore.getState();
    for (let i = 1; i <= 7; i++) {
      store.upsertConversationFromMessage({
        accountIndex: i <= 6 ? 1 : 2,
        senderName: `User ${i}`,
        body: `Msg ${i}`,
        threadId: `thread_${i}`,
        timestamp: Date.now() + i,
      });
      store.togglePinConversation(`thread_${i}`);
    }
    const pinnedCount = useConversationStore.getState().conversations.filter((c) => c.isPinned).length;
    expect(pinnedCount).toBe(6);
    store.clearAccountConversations(1);
    const remaining = useConversationStore.getState().conversations;
    expect(remaining).toHaveLength(1);
    expect(remaining[0].accountIndex).toBe(2);
  });
});
