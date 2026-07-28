import type { SiteCompositionResponse } from './site-composition-response';

export interface SitesCompositionUpdateResponse {
  code: 0;
  data: unknown & { item: SiteCompositionResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
