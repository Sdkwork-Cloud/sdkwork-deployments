import type { SourceRepositoryResponse } from './source-repository-response';

export interface SourceRepositoriesRetrieveResponse {
  code: 0;
  data: unknown & { item: SourceRepositoryResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
