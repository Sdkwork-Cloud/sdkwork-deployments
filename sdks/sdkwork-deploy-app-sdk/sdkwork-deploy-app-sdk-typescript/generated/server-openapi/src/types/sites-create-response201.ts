import type { SiteResponse } from './site-response';

export interface SitesCreateResponse201 {
  code: 0;
  data: unknown & { item: SiteResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
