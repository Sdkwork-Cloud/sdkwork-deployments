export interface BuildQueueItemResponse {
  id: string;
  appId: string;
  platformTargetId: string;
  buildNumber: string;
  buildStatus: string;
  runnerNodeUuid?: string;
  createdAt: string;
  updatedAt: string;
}
