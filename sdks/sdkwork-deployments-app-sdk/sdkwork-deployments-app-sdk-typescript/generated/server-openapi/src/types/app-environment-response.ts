export interface AppEnvironmentResponse {
  id: string;
  appId: string;
  envKey: string;
  envName: string;
  envLevel: string;
  approvalRequired: boolean;
  currentReleaseId?: string;
  currentReleaseVersion?: string;
  envStatus: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
