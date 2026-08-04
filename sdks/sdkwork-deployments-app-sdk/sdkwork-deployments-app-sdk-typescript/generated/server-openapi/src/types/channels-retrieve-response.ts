import type { ChannelResponse } from './channel-response';

export interface ChannelsRetrieveResponse {
  code: 0;
  data: unknown & { item: ChannelResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
