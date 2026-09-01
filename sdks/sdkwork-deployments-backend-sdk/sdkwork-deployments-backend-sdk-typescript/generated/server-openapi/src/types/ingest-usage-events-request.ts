import type { UsageEventIngestItem } from './usage-event-ingest-item';

export interface IngestUsageEventsRequest {
  nodeUuid?: string;
  events: UsageEventIngestItem[];
}
