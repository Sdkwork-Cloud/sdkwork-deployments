import type { ServerResponse } from './server-response';

export interface ServersUpdateResponse {
  code: 0;
  data: unknown & { item: ServerResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
