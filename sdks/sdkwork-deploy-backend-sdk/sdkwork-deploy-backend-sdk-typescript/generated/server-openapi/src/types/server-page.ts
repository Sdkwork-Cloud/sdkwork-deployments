import type { ServerResponse } from './server-response';

export interface ServerPage {
  items?: ServerResponse[];
  total?: string;
}
