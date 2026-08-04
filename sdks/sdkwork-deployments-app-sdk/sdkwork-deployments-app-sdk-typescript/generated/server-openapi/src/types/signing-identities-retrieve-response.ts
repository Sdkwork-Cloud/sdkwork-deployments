import type { SigningIdentityResponse } from './signing-identity-response';

export interface SigningIdentitiesRetrieveResponse {
  code: 0;
  data: unknown & { item: SigningIdentityResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
