export interface CertificateResponse {
  id: string;
  certName: string;
  certificateSource: 'MANAGED' | 'CUSTOM';
  caProfile: 'LETS_ENCRYPT_STAGING' | 'LETS_ENCRYPT_PRODUCTION' | 'CUSTOM';
  preferredKeyAlgorithm: 'RSA' | 'ECDSA';
  identifiers: string[];
  currentVersionId?: string;
  issuer?: string;
  notBefore?: string;
  notAfter?: string;
  autoRenew: boolean;
  renewalStatus: 'NONE' | 'PLANNED' | 'PROCESSING' | 'FAILED';
  status: 'PENDING' | 'ISSUING' | 'ACTIVE' | 'EXPIRED' | 'FAILED' | 'REVOKED';
  createdAt: string;
  updatedAt: string;
  version: string;
}
