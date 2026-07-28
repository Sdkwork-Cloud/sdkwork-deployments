import type { DeploymentResponse } from './deployment-response';

export interface SitesDeploymentsCreateResponse201 {
  code: 0;
  data: unknown & { item: DeploymentResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
