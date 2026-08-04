import type { AppReleaseResponse } from './app-release-response';
import type { PageInfo } from './page-info';

export interface ReleasesListResponse {
  code: 0;
  data: unknown & { items: AppReleaseResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
