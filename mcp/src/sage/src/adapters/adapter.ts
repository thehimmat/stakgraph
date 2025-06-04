import { Message } from "../types/index.js";

export type Adapter = "github" | "none";

export interface ChatAdapter {
  initialize(): Promise<void>;
  sendResponse(chatId: string, message: Message): Promise<void>;
  onMessageReceived(
    callback: (chatId: string, message: Message) => Promise<void>
  ): void;
  // New methods for webhook storage
  storeWebhook(chatId: string, webhook: string): void;
  getWebhook(chatId: string): string | undefined;
}

export abstract class BaseAdapter implements ChatAdapter {
  // Initialize with a no-op async function
  protected messageCallback: (
    chatId: string,
    message: Message
  ) => Promise<void> = async () => {
    /* default no-op implementation */
  };

  // Map to store webhooks for each chat ID
  protected webhooks: Map<string, string> = new Map();

  abstract initialize(): Promise<void>;
  abstract sendResponse(chatId: string, message: Message): Promise<void>;

  onMessageReceived(
    callback: (chatId: string, message: Message) => Promise<void>
  ): void {
    this.messageCallback = callback;
  }

  // Default implementation for webhook storage
  storeWebhook(chatId: string, webhook: string): void {
    this.webhooks.set(chatId, webhook);
    console.log(`Stored webhook for chat ID ${chatId}`);
  }

  // Default implementation for webhook retrieval
  getWebhook(chatId: string): string | undefined {
    return this.webhooks.get(chatId);
  }
}

export class NoAdapter extends BaseAdapter {
  async initialize(): Promise<void> {}
  async sendResponse(_: string, __: Message): Promise<void> {}
}

export function EmptyAdapters(): Record<Adapter, ChatAdapter> {
  return {
    github: new NoAdapter(),
    none: new NoAdapter(),
  };
}