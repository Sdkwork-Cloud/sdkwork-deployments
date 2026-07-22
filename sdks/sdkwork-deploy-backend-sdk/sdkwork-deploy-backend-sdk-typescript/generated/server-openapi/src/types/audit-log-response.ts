export interface AuditLogResponse {
  id?: string;
  operatorId?: string;
  operatorType?: string;
  action?: string;
  targetType?: string;
  targetId?: string;
  targetUuid?: string;
  ipAddress?: string;
  changes?: Record<string, unknown>;
  createdAt?: string;
}
