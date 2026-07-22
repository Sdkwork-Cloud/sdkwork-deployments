export interface ArtifactResponse {
  id?: string;
  siteId?: string;
  packageType?: number;
  fileName?: string;
  contentType?: string;
  contentLength?: string;
  checksumSha256?: string;
  driveNodeId?: string;
  uploadSessionId?: string;
  status?: number;
  createdAt?: string;
}
