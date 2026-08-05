import type { PageInfo } from './page-info';
import type { UsageEventResponse } from './usage-event-response';

export interface UsageEventsListResponse {
  code: 0;
  data: unknown & { items: UsageEventResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
