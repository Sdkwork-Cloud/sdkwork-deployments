import type { Platform } from './platform';
import type { TechStack } from './tech-stack';

export interface BuildTemplateResponse {
  id: string;
  templateName: string;
  templateVersion: string;
  platform: Platform;
  techStack: TechStack;
  toolchain?: Record<string, unknown>;
  commands?: string[];
  artifactOutputs?: string[];
  qualityGates?: Record<string, unknown>;
  templateStatus: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
