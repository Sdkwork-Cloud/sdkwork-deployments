import type { DeploymentKind } from './deployment-kind';
import type { DeploymentTarget } from './deployment-target';
import type { RolloutStrategy } from './rollout-strategy';

export interface CreateAppDeploymentRequest {
  platformTargetId: string;
  releaseId: string;
  deploymentKind: DeploymentKind;
  deploymentTarget: DeploymentTarget;
  environment?: string;
  strategy?: RolloutStrategy;
  percentage?: number;
  idempotencyKey: string;
}
