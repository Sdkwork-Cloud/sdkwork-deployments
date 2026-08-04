import type { BuildTemplateResponse } from './build-template-response';

export interface BuildTemplatesCreateResponse201 {
  code: 0;
  data: unknown & { item: BuildTemplateResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
