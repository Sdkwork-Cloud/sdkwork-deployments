import type { PageInfo } from './page-info';
import type { SigningIdentityHealthResponse } from './signing-identity-health-response';

export interface SigningIdentityHealthListResponse {
  code: 0;
  data: unknown & { items: SigningIdentityHealthResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
