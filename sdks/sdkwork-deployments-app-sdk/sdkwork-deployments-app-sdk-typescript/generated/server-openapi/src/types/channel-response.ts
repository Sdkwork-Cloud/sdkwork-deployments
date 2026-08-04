import type { ChannelKey } from './channel-key';

export interface ChannelResponse {
  id: string;
  appId: string;
  platformTargetId: string;
  channelKey: ChannelKey;
  currentReleaseId?: string;
  currentReleaseVersion?: string;
  channelStatus: string;
  updatedAt: string;
  version: string;
}
