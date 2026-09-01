import type { SourceEventIngestResponse } from './source-event-ingest-response';

export interface CreatePostResponse2012 {
  code: 0;
  data: unknown & { item: SourceEventIngestResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
