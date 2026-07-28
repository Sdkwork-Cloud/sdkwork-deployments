import type { DomainResponse } from './domain-response';

export interface SitesDomainsCreateResponse201 {
  code: 0;
  data: unknown & { item: DomainResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
