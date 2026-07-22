export interface CreateReleaseRequest {
  artifactId: string;
  versionTag?: string;
  idempotencyKey: string;
}
