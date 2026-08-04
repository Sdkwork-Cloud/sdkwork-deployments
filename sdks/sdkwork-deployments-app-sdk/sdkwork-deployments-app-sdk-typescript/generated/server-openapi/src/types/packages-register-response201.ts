import type { PackageResponse } from './package-response';

export interface PackagesRegisterResponse201 {
  code: 0;
  data: unknown & { item: PackageResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
