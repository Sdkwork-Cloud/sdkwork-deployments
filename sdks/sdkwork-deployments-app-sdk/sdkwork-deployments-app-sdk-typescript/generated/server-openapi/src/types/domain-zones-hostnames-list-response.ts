import type { DomainHostnameResponse } from './domain-hostname-response';
import type { PageInfo } from './page-info';

export interface DomainZonesHostnamesListResponse {
  code: 0;
  data: unknown & { items: DomainHostnameResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
