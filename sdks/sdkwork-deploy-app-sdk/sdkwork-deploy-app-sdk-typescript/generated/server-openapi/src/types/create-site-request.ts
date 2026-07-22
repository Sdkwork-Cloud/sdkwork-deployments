export interface CreateSiteRequest {
  name: string;
  slug?: string;
  description?: string;
  siteType: 1 | 2 | 3 | 4 | 5 | 6;
  runtimeConfig?: { buildCommand?: string; outputDirectory?: string; nodeVersion?: string; installCommand?: string; startCommand?: string; };
}
