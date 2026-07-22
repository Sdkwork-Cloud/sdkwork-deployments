import type { NginxReloadResponse } from './nginx-reload-response';

export interface RuntimeReloadResponse {
  code: 0;
  data: unknown & { item: NginxReloadResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
