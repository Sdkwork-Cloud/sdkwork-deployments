export interface EntitlementProjectionResponse {
  id: string;
  tenantId: string;
  sourceSystem: string;
  sourceSubscriptionUuid: string;
  sourceRevision?: string;
  planKey?: string;
  /** dimension limits keyed by snake_case dimension names (active_apps, build_concurrency, package_storage_bytes, ...) */
  entitlements: Record<string, unknown>;
  effectiveAt: string;
  expiresAt?: string;
  projectionStatus: string;
  createdAt: string;
  updatedAt: string;
}
