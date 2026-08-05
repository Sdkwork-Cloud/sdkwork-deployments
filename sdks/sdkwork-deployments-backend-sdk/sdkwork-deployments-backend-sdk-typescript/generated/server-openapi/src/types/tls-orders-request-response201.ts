import type { CertificateOrderResponse } from './certificate-order-response';

export interface TlsOrdersRequestResponse201 {
  code: 0;
  data: unknown & { item: CertificateOrderResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
