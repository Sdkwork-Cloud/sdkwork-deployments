export interface CreateArtifactRequest {
  siteId?: string;
  packageType: number;
  fileName: string;
  contentType: string;
  contentLength: string;
  checksumSha256?: string;
  driveUploadSessionId: string;
  driveUploadItemId?: string;
  driveSpaceId: string;
  driveNodeId: string;
  idempotencyKey: string;
}
