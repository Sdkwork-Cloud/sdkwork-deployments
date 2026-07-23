import {
  createClient as createGeneratedBackendClient,
  SdkworkDeployBackendClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkBackendConfig } from '../generated/server-openapi/src/types/common';

export {
  SdkworkDeployBackendClient,
  SdkworkDeployBackendClient as SdkworkBackendClient,
  createGeneratedBackendClient,
};
export type { SdkworkBackendConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export function createClient(config: SdkworkBackendConfig): SdkworkDeployBackendClient {
  return createGeneratedBackendClient(config);
}
