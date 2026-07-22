import {
  createClient as createGeneratedAppClient,
  SdkworkDeployAppClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';

export {
  SdkworkDeployAppClient,
  SdkworkDeployAppClient as SdkworkAppClient,
  createGeneratedAppClient,
};
export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export function createClient(config: SdkworkAppConfig): SdkworkDeployAppClient {
  return createGeneratedAppClient(config);
}
