import type { CertificateChallengeResponse } from './certificate-challenge-response';
import type { PageInfo } from './page-info';

export interface TlsOrdersListChallengesResponse {
  code: 0;
  data: unknown & { items: CertificateChallengeResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
