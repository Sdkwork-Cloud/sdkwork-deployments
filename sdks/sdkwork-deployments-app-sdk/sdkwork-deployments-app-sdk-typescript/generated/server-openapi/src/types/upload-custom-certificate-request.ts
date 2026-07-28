export interface UploadCustomCertificateRequest {
  certName: string;
  siteId?: string;
  domainId?: string;
  certificateUploadSessionId: string;
  privateKeyUploadSessionId: string;
  idempotencyKey: string;
}
