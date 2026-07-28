#!/usr/bin/env node

import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const scriptRoot = path.dirname(fileURLToPath(import.meta.url));
const familyRoot = path.resolve(scriptRoot, '..');
const applicationRoot = path.resolve(familyRoot, '..', '..');
const workspaceRoot = path.resolve(applicationRoot, '..');
const generator = path.join(workspaceRoot, 'sdkwork-sdk-generator', 'bin', 'sdkgen.js');
const output = path.join(
  familyRoot,
  'sdkwork-deployments-backend-sdk-typescript',
  'generated',
  'server-openapi',
);
const materialize = spawnSync(process.execPath, [
  path.join(applicationRoot, 'tools', 'materialize_deploy_phase1_contracts.mjs'),
], { cwd: applicationRoot, stdio: 'inherit' });
if (materialize.status !== 0) process.exit(materialize.status ?? 1);

const generated = spawnSync(process.execPath, [
  generator,
  'generate',
  '-i', path.join(familyRoot, 'openapi', 'deploy-backend-api.sdkgen.json'),
  '-o', output,
  '-n', 'sdkwork-deployments-backend-sdk',
  '-t', 'backend',
  '-l', 'typescript',
  '--fixed-sdk-version', '0.1.0',
  '--base-url', 'http://127.0.0.1:3900',
  '--api-prefix', '/backend/v3/api',
  '--package-name', '@sdkwork/deployments-backend-sdk',
  '--client-name', 'SdkworkDeployBackendClient',
  '--standard-profile', 'sdkwork-v3',
  '--sdk-root', familyRoot,
  '--sdk-name', 'sdkwork-deployments-backend-sdk',
  '--no-sync-published-version',
], { cwd: applicationRoot, stdio: 'inherit' });
process.exit(generated.status ?? 1);
