import type { AppDeploymentResponse } from './app-deployment-response';
import type { PageInfo } from './page-info';

export interface DeploymentsListResponse {
  code: 0;
  data: unknown & { items: AppDeploymentResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
