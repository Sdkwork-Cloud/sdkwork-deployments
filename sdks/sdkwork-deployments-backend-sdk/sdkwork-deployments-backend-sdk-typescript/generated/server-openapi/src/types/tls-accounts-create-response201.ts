import type { AcmeAccountResponse } from './acme-account-response';

export interface TlsAccountsCreateResponse201 {
  code: 0;
  data: unknown & { item: AcmeAccountResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
