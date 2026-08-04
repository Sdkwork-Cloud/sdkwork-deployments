import type { SigningKind } from './signing-kind';

export interface CreateSigningIdentityRequest {
  identityName: string;
  signingKind: SigningKind;
  platformTargetId?: string;
  fingerprintSha256?: string;
  expiresAt?: string;
  secretRef?: string;
  idempotencyKey?: string;
}
