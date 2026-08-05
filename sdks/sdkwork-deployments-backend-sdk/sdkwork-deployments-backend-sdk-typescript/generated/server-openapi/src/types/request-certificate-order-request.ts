export interface RequestCertificateOrderRequest {
  certificateId: string;
  idempotencyKey: string;
  challengeType?: 'HTTP_01' | 'DNS_01';
}
