export interface SiteSecurityPolicy {
  forceHttps?: boolean;
  denyDotFiles?: boolean;
  deniedPathPrefixes?: string[];
}
