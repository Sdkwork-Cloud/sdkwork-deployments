import type { ArtifactResponse } from './artifact-response';

export interface ArtifactsCreateResponse201 {
  code: 0;
  data: unknown & { item: ArtifactResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
