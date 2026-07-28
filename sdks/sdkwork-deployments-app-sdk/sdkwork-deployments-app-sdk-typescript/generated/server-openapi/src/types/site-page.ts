import type { SiteResponse } from './site-response';

export interface SitePage {
  items?: SiteResponse[];
  total?: string;
  page?: number;
  pageSize?: number;
}
