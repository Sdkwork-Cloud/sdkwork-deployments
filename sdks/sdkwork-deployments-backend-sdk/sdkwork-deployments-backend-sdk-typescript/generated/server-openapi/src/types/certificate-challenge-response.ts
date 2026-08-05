export interface CertificateChallengeResponse {
  id: string;
  tenantId: string;
  orderId: string;
  identifierId: string;
  hostname: string;
  challengeType: string;
  proofSha256: string;
  presentationRef?: string;
  status: string;
  attemptCount: number;
  checkedAt?: string;
  validatedAt?: string;
  lastErrorCode?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
