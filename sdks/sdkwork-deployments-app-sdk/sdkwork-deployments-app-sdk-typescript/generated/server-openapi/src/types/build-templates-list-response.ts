import type { BuildTemplateResponse } from './build-template-response';
import type { PageInfo } from './page-info';

export interface BuildTemplatesListResponse {
  code: 0;
  data: unknown & { items: BuildTemplateResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
