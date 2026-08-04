import type { PageInfo } from './page-info';
import type { SourceRepositoryResponse } from './source-repository-response';

export interface SourceRepositoriesListResponse {
  code: 0;
  data: unknown & { items: SourceRepositoryResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
