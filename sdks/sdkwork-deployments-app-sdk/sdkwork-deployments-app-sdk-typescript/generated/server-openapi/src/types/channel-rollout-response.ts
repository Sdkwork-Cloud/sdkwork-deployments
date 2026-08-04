import type { RolloutStatus } from './rollout-status';
import type { RolloutStrategy } from './rollout-strategy';

export interface ChannelRolloutResponse {
  id: string;
  channelId: string;
  releaseId: string;
  releaseVersion: string;
  strategy: RolloutStrategy;
  percentage?: number;
  rolloutStatus: RolloutStatus;
  supersedesRolloutId?: string;
  requestedAt: string;
  completedAt?: string;
}
