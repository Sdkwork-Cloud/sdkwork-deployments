export interface SourceEventResponse {
  id: string;
  tenantId: string;
  appId: string;
  sourceRepositoryId: string;
  eventKind: string;
  sourceRef: string;
  sourceCommit: string;
  commitMessage?: string;
  payloadSha256: string;
  eventStatus: string;
  buildsTriggered: number;
  errorCode?: string;
  processedAt?: string;
  createdAt: string;
}
