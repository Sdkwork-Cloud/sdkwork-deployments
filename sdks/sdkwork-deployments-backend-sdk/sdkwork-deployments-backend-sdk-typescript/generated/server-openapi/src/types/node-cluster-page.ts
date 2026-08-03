import type { NodeClusterResponse } from './node-cluster-response';

export interface NodeClusterPage {
  items?: NodeClusterResponse[];
  total?: string;
}
