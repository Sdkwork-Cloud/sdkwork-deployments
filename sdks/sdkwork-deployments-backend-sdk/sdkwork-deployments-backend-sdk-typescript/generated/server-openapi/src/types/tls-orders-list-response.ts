import type { CertificateOrderResponse } from './certificate-order-response';
import type { PageInfo } from './page-info';

export interface TlsOrdersListResponse {
  code: 0;
  data: unknown & { items: CertificateOrderResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
