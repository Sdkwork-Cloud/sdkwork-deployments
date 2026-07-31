export interface DomainHostnameResponse {
  id: string;
  zoneId: string;
  hostname: string;
  relativeName: string;
  hostnameType: 'EXACT' | 'WILDCARD';
  verificationStatus: 'PENDING' | 'VERIFIED' | 'FAILED' | 'EXPIRED';
  verifiedAt?: string;
  status: 'ACTIVE' | 'PAUSED';
  certificateCount: string;
  bindingCount: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
