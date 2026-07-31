import type { DomainZoneResponse } from './domain-zone-response';

export interface DomainZonesUpdateResponse {
  code: 0;
  data: unknown & { item: DomainZoneResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
