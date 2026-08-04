import type { BuildResponse } from './build-response';

export interface BuildsRetrieveResponse {
  code: 0;
  data: unknown & { item: BuildResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
