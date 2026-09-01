import type { UsageEventAttribution } from './usage-event-attribution';

export interface UsageEventIngestItem {
  tenantId: string;
  organizationId?: string;
  siteUuid?: string;
  bindingUuid?: string;
  periodStart: string;
  dimension: string;
  quantity: string;
  unit: string;
  deduplicationKey: string;
  attribution: UsageEventAttribution;
  observedAt?: string;
}
