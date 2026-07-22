import type { ReleaseResponse } from './release-response';

export interface ReleasePage {
  items?: ReleaseResponse[];
  total?: string;
}
