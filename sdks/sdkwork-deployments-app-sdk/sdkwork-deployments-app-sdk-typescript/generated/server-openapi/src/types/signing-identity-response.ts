import type { SigningKind } from './signing-kind';

export interface SigningIdentityResponse {
  id: string;
  identityName: string;
  signingKind: SigningKind;
  platformTargetId?: string;
  fingerprintSha256?: string;
  expiresAt?: string;
  secretRef?: string;
  identityStatus: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
