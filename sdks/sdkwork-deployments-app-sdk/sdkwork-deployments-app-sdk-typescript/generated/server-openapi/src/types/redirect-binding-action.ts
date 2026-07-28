export interface RedirectBindingAction {
  type: 'REDIRECT';
  statusCode: 301 | 302 | 307 | 308;
  scheme: 'http' | 'https';
  hostname: string;
  pathPrefix: string;
  preservePath?: boolean;
  preserveQuery?: boolean;
}
