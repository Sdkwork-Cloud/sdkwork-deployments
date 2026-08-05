import type { AcmeAccountResponse } from './acme-account-response';
import type { PageInfo } from './page-info';

export interface TlsAccountsListResponse {
  code: 0;
  data: unknown & { items: AcmeAccountResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
