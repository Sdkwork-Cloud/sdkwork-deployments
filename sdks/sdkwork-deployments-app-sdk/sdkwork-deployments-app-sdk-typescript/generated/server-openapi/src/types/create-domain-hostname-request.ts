export interface CreateDomainHostnameRequest {
  /** Use @ for the apex, a relative label path such as www or api.eu, or * for a wildcard hostname. */
  relativeName: string;
}
