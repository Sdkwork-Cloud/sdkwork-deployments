import type { AuditLogResponse } from './audit-log-response';

export interface AuditLogPage {
  items?: AuditLogResponse[];
  total?: string;
}
