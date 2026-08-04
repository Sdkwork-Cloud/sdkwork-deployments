import type { AppReleaseResponse } from './app-release-response';

export interface ReleasesCreateResponse201 {
  code: 0;
  data: unknown & { item: AppReleaseResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
