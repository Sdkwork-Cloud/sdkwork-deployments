import type { CompositionKey } from './composition-key';
import type { SiteBindingDefinition } from './site-binding-definition';
import type { SiteDeliveryPolicy } from './site-delivery-policy';
import type { SiteEnvironment } from './site-environment';
import type { SiteMountDefinition } from './site-mount-definition';
import type { SiteObservabilityPolicy } from './site-observability-policy';
import type { SiteResourceDefinition } from './site-resource-definition';
import type { SiteRuntimeLimits } from './site-runtime-limits';
import type { SiteSecurityPolicy } from './site-security-policy';
import type { SiteVariantDefinition } from './site-variant-definition';
import type { SiteVariantRuleDefinition } from './site-variant-rule-definition';

export interface UpdateSiteCompositionRequest {
  environment: SiteEnvironment;
  defaultVariantKey: CompositionKey;
  resources: SiteResourceDefinition[];
  variants: SiteVariantDefinition[];
  variantRules?: SiteVariantRuleDefinition[];
  mounts: SiteMountDefinition[];
  bindings: SiteBindingDefinition[];
  deliveryPolicy?: SiteDeliveryPolicy;
  securityPolicy?: SiteSecurityPolicy;
  limits?: SiteRuntimeLimits;
  observabilityPolicy?: SiteObservabilityPolicy;
}
