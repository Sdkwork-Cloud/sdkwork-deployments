export interface SourceRepositoryResponse {
  id: string;
  appId: string;
  repoKey: string;
  repoProvider: string;
  repoUrl: string;
  defaultBranch: string;
  cloneMode: string;
  credentialSecretRef?: string;
  repoStatus: string;
  lastErrorCode?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
