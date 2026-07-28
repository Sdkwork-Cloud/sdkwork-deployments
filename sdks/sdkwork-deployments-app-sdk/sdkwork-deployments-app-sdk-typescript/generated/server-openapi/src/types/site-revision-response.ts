export interface SiteRevisionResponse {
  id: string;
  number: string;
  descriptorSha256: string;
  validationStatus: 'VALID' | 'INVALID';
}
