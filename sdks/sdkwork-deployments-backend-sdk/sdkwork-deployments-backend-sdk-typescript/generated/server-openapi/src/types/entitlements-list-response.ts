import type { EntitlementProjectionResponse } from './entitlement-projection-response';
import type { PageInfo } from './page-info';

export interface EntitlementsListResponse {
  code: 0;
  data: unknown & { items: EntitlementProjectionResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
