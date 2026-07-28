import type { DriveFolderSelector } from './drive-folder-selector';
import type { DriveSpaceRootSelector } from './drive-space-root-selector';

export interface DriveDirectorySource {
  type: 'DRIVE_DIRECTORY';
  websiteSpaceId: string;
  root: DriveSpaceRootSelector | DriveFolderSelector;
  contentMode: 'LIVE_TREE' | 'ATOMIC_GENERATION';
}
