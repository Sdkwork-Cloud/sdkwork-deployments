import type { EnvironmentPromotionResponse } from './environment-promotion-response';

export interface AppEnvironmentsCreatePostResponse201 {
  code: 0;
  data: unknown & { item: EnvironmentPromotionResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
