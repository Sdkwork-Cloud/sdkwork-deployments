export interface UsageEventResponse {
  id: string;
  tenantId: string;
  siteId?: string;
  periodStart: string;
  /** usage dimension (build_minutes | package_storage_bytes | deployment_count) */
  dimension: string;
  quantity: string;
  unit: string;
  sourceTargetUuid?: string;
  sourceWindowId?: string;
  deduplicationKey: string;
  observedAt: string;
  createdAt: string;
}
