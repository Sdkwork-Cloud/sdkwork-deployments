export interface CreateDeployUploadSessionRequest {
  siteId?: string;
  /** 1=archive, 2=static, 3=docker, 4=jar, 5=war, 6=tls_certificate, 7=tls_private_key */
  packageType: number;
  fileName: string;
  contentType: string;
  contentLength: string;
  checksum?: string;
  idempotencyKey: string;
}
