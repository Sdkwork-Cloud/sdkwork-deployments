export interface CreateDomainHostnameRequest {
  /** Relative name within the zone: "@" addresses the apex (already part of the zone), a label path such as www or api.eu creates multi-level subdomains, and "*" or "*.a" creates wildcard hostnames (the "*" label is accepted only as the leftmost label). Names are ASCII-only after IDNA conversion; leading/trailing dots, empty labels, and hyphen-edged labels are rejected.
 */
  relativeName: string;
}
