import type { BuildQueueItemResponse } from './build-queue-item-response';
import type { PageInfo } from './page-info';

export interface BuildQueueListResponse {
  code: 0;
  data: unknown & { items: BuildQueueItemResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
