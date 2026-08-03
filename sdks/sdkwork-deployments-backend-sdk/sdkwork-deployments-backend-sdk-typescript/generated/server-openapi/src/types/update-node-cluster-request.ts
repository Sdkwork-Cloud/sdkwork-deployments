export interface UpdateNodeClusterRequest {
  description?: string;
  region?: string;
  /** 状态：0=启用，1=停用 */
  status?: 0 | 1;
}
