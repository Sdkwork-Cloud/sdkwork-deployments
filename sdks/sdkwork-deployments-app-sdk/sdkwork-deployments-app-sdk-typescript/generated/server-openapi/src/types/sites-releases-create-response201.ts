import type { ReleaseResponse } from './release-response';

export interface SitesReleasesCreateResponse201 {
  code: 0;
  data: unknown & { item: ReleaseResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
