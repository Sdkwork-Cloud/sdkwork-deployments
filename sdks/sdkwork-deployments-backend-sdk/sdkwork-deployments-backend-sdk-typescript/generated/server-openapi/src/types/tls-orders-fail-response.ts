export interface TlsOrdersFailResponse {
  code: 0;
  data: unknown & { item: Record<string, unknown>; };
  /** Server-owned request correlation id. */
  traceId: string;
}
