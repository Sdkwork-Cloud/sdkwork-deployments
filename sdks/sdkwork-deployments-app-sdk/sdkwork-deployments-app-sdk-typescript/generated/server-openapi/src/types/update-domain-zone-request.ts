export interface UpdateDomainZoneRequest {
  displayName?: string;
  dnsProvider?: string;
  providerZoneRef?: string;
  status?: 'ACTIVE' | 'PAUSED';
}
