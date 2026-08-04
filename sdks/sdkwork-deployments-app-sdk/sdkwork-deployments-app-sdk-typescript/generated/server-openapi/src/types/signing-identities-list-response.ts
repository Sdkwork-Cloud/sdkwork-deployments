import type { PageInfo } from './page-info';
import type { SigningIdentityResponse } from './signing-identity-response';

export interface SigningIdentitiesListResponse {
  code: 0;
  data: unknown & { items: SigningIdentityResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
