import type { SourceRepositoryResponse } from './source-repository-response';

export interface SourceRepositoriesCreateResponse201 {
  code: 0;
  data: unknown & { item: SourceRepositoryResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
