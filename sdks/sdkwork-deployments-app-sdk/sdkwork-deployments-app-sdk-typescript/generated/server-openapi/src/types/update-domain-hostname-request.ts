export interface UpdateDomainHostnameRequest {
  /** New relative name for the hostname, with the same syntax as create. Renaming invalidates the ownership proof (verification returns to PENDING) and is rejected while the hostname is bound to an application or a certificate, or when it targets the zone apex.
 */
  relativeName: string;
}
