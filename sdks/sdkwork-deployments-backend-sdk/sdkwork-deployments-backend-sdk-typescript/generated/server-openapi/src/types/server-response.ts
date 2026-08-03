export interface ServerResponse {
  id?: string;
  name?: string;
  host?: string;
  sshPort?: number;
  clusterId?: string;
  clusterName?: string;
  /** 节点角色：0=宿主节点，1=边缘节点 */
  nodeRole?: number;
  status?: number;
  sshUser?: string;
  description?: string;
  createdAt?: string;
}
