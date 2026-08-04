import type { BuildResponse } from './build-response';
import type { PageInfo } from './page-info';

export interface BuildsListResponse {
  code: 0;
  data: unknown & { items: BuildResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
