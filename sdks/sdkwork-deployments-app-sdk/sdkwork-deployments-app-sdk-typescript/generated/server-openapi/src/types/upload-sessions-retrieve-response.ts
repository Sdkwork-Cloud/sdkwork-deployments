import type { DeployUploadSessionResponse } from './deploy-upload-session-response';

export interface UploadSessionsRetrieveResponse {
  code: 0;
  data: unknown & { item: DeployUploadSessionResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
