export interface DeploymentResponse {
  id?: string;
  siteId?: string;
  deployType?: number;
  releaseId?: string;
  versionTag?: string;
  status?: number;
  startedAt?: string;
  completedAt?: string;
  durationMs?: string;
  createdAt?: string;
}
