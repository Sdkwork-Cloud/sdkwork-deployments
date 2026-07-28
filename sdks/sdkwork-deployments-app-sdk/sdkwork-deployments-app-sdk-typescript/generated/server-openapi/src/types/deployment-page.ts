import type { DeploymentResponse } from './deployment-response';

export interface DeploymentPage {
  items?: DeploymentResponse[];
  total?: string;
}
