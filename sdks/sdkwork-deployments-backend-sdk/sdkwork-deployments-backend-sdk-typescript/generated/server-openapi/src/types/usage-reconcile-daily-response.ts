import type { UsageReconciliationResponse } from './usage-reconciliation-response';

export interface UsageReconcileDailyResponse {
  code: 0;
  data: unknown & { item: UsageReconciliationResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
