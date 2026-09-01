import type { PageInfo } from './page-info';
import type { SourceEventResponse } from './source-event-response';

export interface ListGetResponse2 {
  code: 0;
  data: unknown & { items: SourceEventResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
