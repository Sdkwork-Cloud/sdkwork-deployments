import type { ServerResponse } from './server-response';

export interface ServersCreateResponse201 {
  code: 0;
  data: unknown & { item: ServerResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
