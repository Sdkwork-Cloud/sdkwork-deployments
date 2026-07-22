import type { CompositionKey } from './composition-key';

export interface SiteMountDefinition {
  key: CompositionKey;
  variantKey: CompositionKey;
  resourceKey: CompositionKey;
  pathPrefix: string;
  resourceSubpath: string;
  mode: 'ROOT' | 'ALIAS';
  handler: 'STATIC' | 'SPA' | 'WIKI';
  indexFiles?: string[];
  spaFallback?: string | null;
  priority?: number;
}
