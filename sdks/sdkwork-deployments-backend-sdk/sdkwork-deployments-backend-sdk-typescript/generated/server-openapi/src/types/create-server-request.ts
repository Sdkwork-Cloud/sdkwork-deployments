export interface CreateServerRequest {
  name: string;
  host: string;
  sshPort: number;
  /** 所属节点集群 */
  clusterId?: string;
  sshUser?: string;
  sshKeyPath?: string;
  description?: string;
}
