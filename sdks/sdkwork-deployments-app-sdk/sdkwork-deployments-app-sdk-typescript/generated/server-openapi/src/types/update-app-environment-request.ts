export interface UpdateAppEnvironmentRequest {
  envName?: string;
  approvalRequired?: boolean;
  envStatus?: 'DRAFT' | 'ACTIVE' | 'ARCHIVED';
}
