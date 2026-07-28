import type { SiteResponse } from './site-response';

export interface SitesRetrieveResponse {
  code: 0;
  data: unknown & { item: SiteResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
