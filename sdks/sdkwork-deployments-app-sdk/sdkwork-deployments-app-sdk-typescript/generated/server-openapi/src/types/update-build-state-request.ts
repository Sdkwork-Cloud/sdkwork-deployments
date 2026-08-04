import type { BuildStatus } from './build-status';

export interface UpdateBuildStateRequest {
  buildStatus: BuildStatus;
  runnerNodeUuid: string;
  runnerVersion?: string;
  logRef?: string;
  sourceSnapshot?: Record<string, unknown>;
  qualityGate?: Record<string, unknown>;
  errorCode?: string;
  startedAt?: string;
  finishedAt?: string;
}
