export interface SiteRuntimeAssignmentResponse {
  targetId: string;
  assignmentId: string;
  generation: string;
  status: 'PENDING' | 'PUBLISHING' | 'PUBLISHED' | 'FAILED' | 'SUPERSEDED';
}
