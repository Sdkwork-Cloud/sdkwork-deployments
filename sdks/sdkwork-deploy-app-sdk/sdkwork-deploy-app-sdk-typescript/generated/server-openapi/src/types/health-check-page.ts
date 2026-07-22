import type { HealthCheckResponse } from './health-check-response';

export interface HealthCheckPage {
  items?: HealthCheckResponse[];
  total?: string;
}
