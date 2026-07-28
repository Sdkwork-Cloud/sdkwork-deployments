import type { DeploymentResponse } from './deployment-response';

export interface SitesDeploymentsRetrieveResponse {
  code: 0;
  data: unknown & { item: DeploymentResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
