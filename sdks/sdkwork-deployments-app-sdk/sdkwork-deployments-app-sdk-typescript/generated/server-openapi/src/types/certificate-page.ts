import type { CertificateResponse } from './certificate-response';

export interface CertificatePage {
  items?: CertificateResponse[];
  total?: string;
}
