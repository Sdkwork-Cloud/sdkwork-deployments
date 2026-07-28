import type { SiteRevisionResponse } from './site-revision-response';
import type { SiteRuntimeAssignmentResponse } from './site-runtime-assignment-response';

export interface SiteCompositionResponse {
  siteId: string;
  siteVersion: string;
  revision: SiteRevisionResponse;
  runtimeAssignments: SiteRuntimeAssignmentResponse[];
}
