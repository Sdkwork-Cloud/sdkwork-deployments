export interface CertificateOrderResponse {
  id: string;
  tenantId: string;
  certificateId: string;
  acmeAccountId: string;
  requestedVersionNo: string;
  requestSha256: string;
  idempotencyKey: string;
  externalOrderDigest?: string;
  status: string;
  attemptCount: number;
  lastErrorCode?: string;
  deadlineAt: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
