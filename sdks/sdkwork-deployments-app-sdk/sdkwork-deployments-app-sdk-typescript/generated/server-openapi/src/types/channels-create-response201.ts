import type { ChannelRolloutResponse } from './channel-rollout-response';

export interface ChannelsCreateResponse201 {
  code: 0;
  data: unknown & { item: ChannelRolloutResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
