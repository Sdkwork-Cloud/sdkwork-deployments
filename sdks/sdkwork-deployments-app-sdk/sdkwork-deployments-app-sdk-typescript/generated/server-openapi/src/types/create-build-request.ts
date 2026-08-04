export interface CreateBuildRequest {
  platformTargetId: string;
  sourceRepositoryId?: string;
  sourceRef?: string;
  templateId?: string;
  semanticVersion?: string;
  idempotencyKey: string;
}
