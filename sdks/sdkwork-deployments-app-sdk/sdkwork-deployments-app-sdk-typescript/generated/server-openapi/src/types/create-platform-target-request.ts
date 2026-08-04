import type { ChannelKey } from './channel-key';
import type { Platform } from './platform';
import type { TechStack } from './tech-stack';

export interface CreatePlatformTargetRequest {
  targetKey: string;
  platform: Platform;
  techStack?: TechStack;
  bundleId?: string;
  packageName?: string;
  appId?: string;
  bundleName?: string;
  buildTemplateId?: string;
  allowedChannels?: ChannelKey[];
  idempotencyKey?: string;
}
