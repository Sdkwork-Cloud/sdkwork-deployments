import type { ReleaseStatus } from './release-status';

export interface CreateAppReleaseRequest {
  platformTargetId: string;
  packageId: string;
  semanticVersion: string;
  releaseNotes?: Record<string, unknown>;
  releaseStatus?: ReleaseStatus;
  idempotencyKey: string;
}
