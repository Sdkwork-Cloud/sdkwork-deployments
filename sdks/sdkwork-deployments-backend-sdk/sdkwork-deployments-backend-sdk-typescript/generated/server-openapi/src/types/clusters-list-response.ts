import type { NodeClusterResponse } from './node-cluster-response';
import type { PageInfo } from './page-info';

export interface ClustersListResponse {
  code: 0;
  data: unknown & { items: NodeClusterResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
