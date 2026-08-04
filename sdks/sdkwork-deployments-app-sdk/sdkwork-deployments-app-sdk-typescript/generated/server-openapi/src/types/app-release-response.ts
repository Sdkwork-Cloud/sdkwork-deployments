import type { ReleaseStatus } from './release-status';

export interface AppReleaseResponse {
  id: string;
  appId: string;
  platformTargetId: string;
  packageId: string;
  semanticVersion: string;
  buildNumber: string;
  releaseStatus: ReleaseStatus;
  releaseNotes?: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
  version: string;
}
