import type { PageInfo } from './page-info';
import type { RunnerHealthResponse } from './runner-health-response';

export interface RunnersListResponse {
  code: 0;
  data: unknown & { items: RunnerHealthResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
