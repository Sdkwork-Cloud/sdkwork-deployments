export interface CreateDomainZoneRequest {
  apexHostname: string;
  displayName?: string;
  dnsProvider?: string;
  providerZoneRef?: string;
}
