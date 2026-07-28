import type { ArtifactResponse } from './artifact-response';
import type { PageInfo } from './page-info';

export interface ArtifactsListResponse {
  code: 0;
  data: unknown & { items: ArtifactResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
