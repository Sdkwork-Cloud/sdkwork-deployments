import type { PackageResponse } from './package-response';
import type { PageInfo } from './page-info';

export interface PackagesListResponse {
  code: 0;
  data: unknown & { items: PackageResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
