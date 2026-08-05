export interface CreateAcmeAccountRequest {
  caProfile: 'LETS_ENCRYPT_STAGING' | 'LETS_ENCRYPT_PRODUCTION';
  directoryUrl: string;
  contactEmail: string;
  externalAccountDigest?: string;
}
