import type { PageInfo } from './page-info';
import type { SiteResponse } from './site-response';

export interface SitesListResponse {
  code: 0;
  data: unknown & { items: SiteResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
