import type { BuildResponse } from './build-response';

export interface BuildsStateUpdateResponse {
  code: 0;
  data: unknown & { item: BuildResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
