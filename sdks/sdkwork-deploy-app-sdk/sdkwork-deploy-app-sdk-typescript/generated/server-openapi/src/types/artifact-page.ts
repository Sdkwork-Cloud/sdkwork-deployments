import type { ArtifactResponse } from './artifact-response';

export interface ArtifactPage {
  items?: ArtifactResponse[];
  total?: string;
}
