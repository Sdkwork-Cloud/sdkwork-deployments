export interface AcmeAccountResponse {
  id: string;
  tenantId: string;
  caProfile: string;
  directoryUrl: string;
  contactEmail: string;
  externalAccountDigest?: string;
  status: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
