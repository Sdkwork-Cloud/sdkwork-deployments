import type { PackageResponse } from './package-response';

export interface PackagesRetrieveResponse {
  code: 0;
  data: unknown & { item: PackageResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
