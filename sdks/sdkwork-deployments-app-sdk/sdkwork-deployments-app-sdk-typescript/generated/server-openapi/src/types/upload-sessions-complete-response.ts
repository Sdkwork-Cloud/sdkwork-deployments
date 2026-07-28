import type { DeployUploadSessionResponse } from './deploy-upload-session-response';

export interface UploadSessionsCompleteResponse {
  code: 0;
  data: unknown & { item: DeployUploadSessionResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
