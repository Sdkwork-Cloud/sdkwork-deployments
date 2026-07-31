export interface DomainVerifyResponse {
  verified: boolean;
  method: 'DNS_TXT';
  /** 当前所有权验证尝试标识；域名已验证时可省略。 */
  verificationId?: string;
  /** 需要配置 TXT 记录的完整规范名称；域名已验证时可省略。 */
  recordName?: string;
  /** 仅在创建验证尝试时返回一次的明文 proof；后续查询和验证响应不会再次返回。 */
  token?: string;
  /** 当前验证尝试的失效时间；域名已验证时可省略。 */
  expiresAt?: string;
}
