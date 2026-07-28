export interface ServeBindingAction {
  type: 'SERVE';
  defaultVariantKey?: string | null;
  forcedVariantKey?: string | null;
}
