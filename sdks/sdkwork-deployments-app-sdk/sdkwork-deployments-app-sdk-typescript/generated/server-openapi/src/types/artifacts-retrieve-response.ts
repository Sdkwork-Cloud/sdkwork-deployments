import type { ArtifactResponse } from './artifact-response';

export interface ArtifactsRetrieveResponse {
  code: 0;
  data: unknown & { item: ArtifactResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
