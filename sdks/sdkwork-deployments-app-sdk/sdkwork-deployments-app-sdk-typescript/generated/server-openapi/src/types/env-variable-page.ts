import type { EnvVariableResponse } from './env-variable-response';

export interface EnvVariablePage {
  items?: EnvVariableResponse[];
  total?: string;
}
