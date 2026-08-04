import type { ChannelRolloutResponse } from './channel-rollout-response';
import type { PageInfo } from './page-info';

export interface ChannelsRolloutsListResponse {
  code: 0;
  data: unknown & { items: ChannelRolloutResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
