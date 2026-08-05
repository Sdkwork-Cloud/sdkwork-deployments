export interface EnvironmentPromotionResponse {
  id: string;
  appId: string;
  environmentId: string;
  environmentKey: string;
  releaseId: string;
  releaseVersion: string;
  fromEnvironmentId?: string;
  fromEnvironmentKey?: string;
  promotedBy?: string;
  note?: string;
  createdAt: string;
}
