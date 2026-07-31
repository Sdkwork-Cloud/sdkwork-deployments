export interface CreateCertificateRequest {
  certName: string;
  domainIds: string[];
  caProfile?: 'LETS_ENCRYPT_STAGING' | 'LETS_ENCRYPT_PRODUCTION';
  preferredKeyAlgorithm?: 'RSA' | 'ECDSA';
}
