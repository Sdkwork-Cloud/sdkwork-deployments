import type { UsageReconciliationResponse } from './usage-reconciliation-response';

export interface CreatePostResponse201 {
  code: 0;
  data: unknown & { item: UsageReconciliationResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
