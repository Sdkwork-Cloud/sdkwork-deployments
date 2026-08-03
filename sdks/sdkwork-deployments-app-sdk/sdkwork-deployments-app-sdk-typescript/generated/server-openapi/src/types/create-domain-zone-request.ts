export interface CreateDomainZoneRequest {
  /** A registrable root domain (zone apex) such as example.com or example.co.uk. Wildcard names and subdomains are not accepted; Unicode (IDN) names are converted to punycode automatically. The apex must not already exist as a hostname and must not overlap an existing root domain.
 */
  apexHostname: string;
  displayName?: string;
  dnsProvider?: string;
  providerZoneRef?: string;
}
