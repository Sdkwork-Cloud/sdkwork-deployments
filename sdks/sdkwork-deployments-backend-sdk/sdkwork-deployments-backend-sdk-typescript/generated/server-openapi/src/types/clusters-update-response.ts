import type { NodeClusterResponse } from './node-cluster-response';

export interface ClustersUpdateResponse {
  code: 0;
  data: unknown & { item: NodeClusterResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
