export interface CreateAppEnvironmentRequest {
  envKey: string;
  envName: string;
  envLevel: 'DEVELOPMENT' | 'STAGING' | 'PRODUCTION';
  approvalRequired?: boolean;
}
