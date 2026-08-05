export interface SigningIdentityHealthResponse {
  id: string;
  tenantId: string;
  identityName: string;
  signingKind: string;
  expiresAt?: string;
  daysUntilExpiry?: string;
  identityStatus: string;
}
