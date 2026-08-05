import type { RetentionRunResponse } from './retention-run-response';

export interface RetentionRunPostResponse {
  code: 0;
  data: unknown & { item: RetentionRunResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
