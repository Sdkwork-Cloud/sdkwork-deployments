import type { DomainResponse } from './domain-response';
import type { PageInfo } from './page-info';

export interface SitesDomainsListResponse {
  code: 0;
  data: unknown & { items: DomainResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
