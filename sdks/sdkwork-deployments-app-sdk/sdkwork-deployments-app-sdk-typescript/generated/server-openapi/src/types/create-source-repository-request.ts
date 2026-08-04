export interface CreateSourceRepositoryRequest {
  repoKey: string;
  repoProvider: 'GITHUB' | 'GITEE' | 'GITLAB' | 'SELF_HOSTED';
  repoUrl: string;
  defaultBranch?: string;
  cloneMode?: 'FULL' | 'SHALLOW';
  credentialSecretRef?: string;
  idempotencyKey?: string;
}
