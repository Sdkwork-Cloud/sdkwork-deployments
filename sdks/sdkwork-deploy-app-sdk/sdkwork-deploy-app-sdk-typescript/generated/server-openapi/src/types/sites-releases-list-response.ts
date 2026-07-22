import type { PageInfo } from './page-info';
import type { ReleaseResponse } from './release-response';

export interface SitesReleasesListResponse {
  code: 0;
  data: unknown & { items: ReleaseResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
