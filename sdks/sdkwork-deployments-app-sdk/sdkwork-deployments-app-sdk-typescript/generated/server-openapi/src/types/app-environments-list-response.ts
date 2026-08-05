import type { AppEnvironmentResponse } from './app-environment-response';
import type { PageInfo } from './page-info';

export interface AppEnvironmentsListResponse {
  code: 0;
  data: unknown & { items: AppEnvironmentResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
