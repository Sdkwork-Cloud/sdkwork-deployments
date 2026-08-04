import type { AppDeploymentResponse } from './app-deployment-response';

export interface DeploymentsRetrieveResponse {
  code: 0;
  data: unknown & { item: AppDeploymentResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
