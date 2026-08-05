import type { EnvironmentPromotionResponse } from './environment-promotion-response';

export interface AppEnvironmentsPromoteResponse201 {
  code: 0;
  data: unknown & { item: EnvironmentPromotionResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
