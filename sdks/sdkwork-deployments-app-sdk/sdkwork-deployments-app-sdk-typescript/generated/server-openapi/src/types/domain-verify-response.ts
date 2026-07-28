export interface DomainVerifyResponse {
  verified: boolean;
  method: 'DNS_TXT';
  /** 待验证时必须写入 _sdkwork-verification.<hostname> 的精确 TXT 值；验证完成后省略。 */
  token?: string;
}
