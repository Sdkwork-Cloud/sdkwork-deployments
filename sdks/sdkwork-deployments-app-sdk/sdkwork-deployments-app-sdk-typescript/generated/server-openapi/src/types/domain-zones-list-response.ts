import type { DomainZoneResponse } from './domain-zone-response';
import type { PageInfo } from './page-info';

export interface DomainZonesListResponse {
  code: 0;
  data: unknown & { items: DomainZoneResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
