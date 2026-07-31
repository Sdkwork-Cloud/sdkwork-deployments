export interface DomainZoneResponse {
  id: string;
  apexHostname: string;
  displayName?: string;
  dnsProvider?: string;
  status: 'ACTIVE' | 'PAUSED';
  hostnameCount: string;
  verifiedHostnameCount: string;
  certificateCount: string;
  bindingCount: string;
  updatedAt: string;
  version: string;
}
