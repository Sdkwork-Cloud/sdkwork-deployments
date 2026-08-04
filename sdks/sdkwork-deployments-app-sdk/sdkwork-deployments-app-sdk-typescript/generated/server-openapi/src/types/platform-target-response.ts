import type { ChannelKey } from './channel-key';
import type { Platform } from './platform';
import type { TechStack } from './tech-stack';

export interface PlatformTargetResponse {
  id: string;
  appId: string;
  targetKey: string;
  platform: Platform;
  techStack: TechStack;
  bundleId?: string;
  packageName?: string;
  appIdValue?: string;
  bundleName?: string;
  buildTemplateId?: string;
  allowedChannels: ChannelKey[];
  targetStatus: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
