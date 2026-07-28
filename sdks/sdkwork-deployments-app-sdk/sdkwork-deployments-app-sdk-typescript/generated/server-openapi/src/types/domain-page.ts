import type { DomainResponse } from './domain-response';

export interface DomainPage {
  items?: DomainResponse[];
  total?: string;
}
