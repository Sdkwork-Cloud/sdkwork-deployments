import type { NginxConfigResponse } from './nginx-config-response';

export interface NginxConfigPage {
  items?: NginxConfigResponse[];
  total?: string;
}
