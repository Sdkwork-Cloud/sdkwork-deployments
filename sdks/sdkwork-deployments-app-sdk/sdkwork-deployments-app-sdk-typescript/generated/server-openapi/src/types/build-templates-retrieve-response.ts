import type { BuildTemplateResponse } from './build-template-response';

export interface BuildTemplatesRetrieveResponse {
  code: 0;
  data: unknown & { item: BuildTemplateResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
