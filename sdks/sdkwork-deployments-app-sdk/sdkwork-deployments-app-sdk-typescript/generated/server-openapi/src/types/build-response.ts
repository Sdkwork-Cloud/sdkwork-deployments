import type { BuildStatus } from './build-status';

export interface BuildResponse {
  id: string;
  appId: string;
  platformTargetId: string;
  templateId?: string;
  buildNumber: string;
  sourceRepositoryId?: string;
  sourceRef?: string;
  sourceSnapshot?: Record<string, unknown>;
  buildStatus: BuildStatus;
  logRef?: string;
  producedPackageId?: string;
  qualityGate?: Record<string, unknown>;
  runnerNodeUuid?: string;
  errorCode?: string;
  startedAt?: string;
  finishedAt?: string;
  durationMs?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
