import type { DeploymentKind } from './deployment-kind';
import type { DeploymentStatus } from './deployment-status';
import type { DeploymentTarget } from './deployment-target';
import type { RolloutStrategy } from './rollout-strategy';

export interface AppDeploymentResponse {
  id: string;
  appId: string;
  platformTargetId?: string;
  siteId?: string;
  releaseId?: string;
  deploymentKind?: DeploymentKind;
  deploymentTarget?: DeploymentTarget;
  environment: string;
  strategy?: RolloutStrategy;
  percentage?: number;
  platformReviewRef?: string;
  deploymentStatus: DeploymentStatus;
  rollbackFromDeploymentId?: string;
  startedAt?: string;
  completedAt?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
