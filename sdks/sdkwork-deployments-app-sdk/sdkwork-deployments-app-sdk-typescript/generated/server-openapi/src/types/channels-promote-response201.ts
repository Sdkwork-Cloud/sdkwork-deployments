import type { ChannelRolloutResponse } from './channel-rollout-response';

export interface ChannelsPromoteResponse201 {
  code: 0;
  data: unknown & { item: ChannelRolloutResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
