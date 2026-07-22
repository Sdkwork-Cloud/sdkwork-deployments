import assert from 'node:assert/strict';
import { parseSdkworkDeployBinding } from '../../../sdkwork-specs/tools/deploy/site-binding.mjs';

const binding = parseSdkworkDeployBinding(
  {
    sdkworkDeploy: {
      appRoot: 'E:/sdkwork-space/sdkwork-im',
      profileId: 'cloud.production',
    },
  },
  'im.sdkwork.com',
);

assert.equal(binding.appRoot, 'E:/sdkwork-space/sdkwork-im');
assert.equal(binding.domain, 'im.sdkwork.com');
assert.equal(binding.profileId, 'cloud.production');

process.stdout.write('deploy-orchestrator.contract.test.mjs passed\n');
