import type { ChannelResponse } from './channel-response';
import type { PageInfo } from './page-info';

export interface ChannelsListResponse {
  code: 0;
  data: unknown & { items: ChannelResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
