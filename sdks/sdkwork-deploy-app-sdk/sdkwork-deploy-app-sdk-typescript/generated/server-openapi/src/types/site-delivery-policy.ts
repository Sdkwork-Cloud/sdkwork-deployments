export interface SiteDeliveryPolicy {
  providerTimeoutMs?: number;
  metadataCacheTtlSeconds?: number;
  negativeCacheTtlSeconds?: number;
  staleWhileRevalidateSeconds?: number;
  maximumObjectBytes?: number;
}
