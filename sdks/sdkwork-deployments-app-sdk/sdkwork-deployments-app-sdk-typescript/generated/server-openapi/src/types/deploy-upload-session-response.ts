export interface DeployUploadSessionResponse {
  id?: string;
  siteId?: string;
  packageType?: number;
  fileName?: string;
  contentType?: string;
  contentLength?: string;
  checksum?: string;
  /** 0=pending, 1=completed, 2=cancelled */
  status?: number;
  driveUploadSessionId?: string;
  driveUploadItemId?: string;
  driveSpaceId?: string;
  driveNodeId?: string;
  createdAt?: string;
  updatedAt?: string;
}
