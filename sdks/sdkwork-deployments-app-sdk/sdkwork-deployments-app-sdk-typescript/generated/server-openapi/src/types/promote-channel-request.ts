import type { RolloutStrategy } from './rollout-strategy';

export interface PromoteChannelRequest {
  releaseId: string;
  strategy?: RolloutStrategy;
  percentage?: number;
  idempotencyKey?: string;
}
