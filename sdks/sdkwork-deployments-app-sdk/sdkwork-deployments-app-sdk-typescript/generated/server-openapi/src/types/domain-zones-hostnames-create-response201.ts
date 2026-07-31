import type { DomainHostnameResponse } from './domain-hostname-response';

export interface DomainZonesHostnamesCreateResponse201 {
  code: 0;
  data: unknown & { item: DomainHostnameResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
