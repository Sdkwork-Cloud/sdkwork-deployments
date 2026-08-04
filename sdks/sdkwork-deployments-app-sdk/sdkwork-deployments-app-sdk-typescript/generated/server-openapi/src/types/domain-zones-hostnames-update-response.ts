import type { DomainHostnameResponse } from './domain-hostname-response';

export interface DomainZonesHostnamesUpdateResponse {
  code: 0;
  data: unknown & { item: DomainHostnameResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
