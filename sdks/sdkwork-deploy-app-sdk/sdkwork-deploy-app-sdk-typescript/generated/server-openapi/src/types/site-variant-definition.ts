import type { CompositionKey } from './composition-key';

export interface SiteVariantDefinition {
  key: CompositionKey;
  label: string;
  clientClass?: 'DESKTOP' | 'MOBILE' | 'TABLET' | 'BOT' | 'OTHER';
  priority?: number;
}
