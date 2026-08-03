export interface UpdateServerRequest {
  name?: string;
  sshPort?: number;
  /** 所属节点集群 */
  clusterId?: string;
  sshUser?: string;
  description?: string;
  /** 状态：0=未连接，1=在线，2=离线，3=维护中 */
  status?: 0 | 1 | 2 | 3;
}
