import type { UsageIngestResult } from './usage-ingest-result';

export interface CreateResponse201 {
  code: 0;
  data: unknown & { item: UsageIngestResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
