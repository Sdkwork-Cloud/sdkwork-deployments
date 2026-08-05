import type { SourceEventIngestResponse } from './source-event-ingest-response';

export interface SourceEventsIngestResponse {
  code: 0;
  data: unknown & { item: SourceEventIngestResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
