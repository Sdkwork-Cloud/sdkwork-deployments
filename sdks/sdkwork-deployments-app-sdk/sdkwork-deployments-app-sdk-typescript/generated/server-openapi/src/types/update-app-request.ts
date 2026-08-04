import type { AppStatus } from './app-status';

export interface UpdateAppRequest {
  name?: string;
  description?: string;
  appStatus?: AppStatus;
  defaultEnvironment?: string;
}
