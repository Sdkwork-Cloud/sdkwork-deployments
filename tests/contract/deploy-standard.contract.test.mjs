#!/usr/bin/env node
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { validateDeploy } from '../../../sdkwork-specs/tools/deploy/validate.mjs';
import { planDeploy } from '../../../sdkwork-specs/tools/deploy/plan.mjs';
import { renderNginxSite } from '../../../sdkwork-specs/tools/deploy/nginx-render.mjs';
import { validateDeploySchema } from '../../../sdkwork-specs/tools/deploy/schema-validate.mjs';
import { inspectNginxConfig } from '../../../sdkwork-specs/tools/deploy/nginx-lifecycle.mjs';
import { resolveDomainSurfaceId } from '../../../sdkwork-specs/tools/deploy/topology-env.mjs';
import { loadTopology } from '../../../sdkwork-specs/tools/deploy/load-manifest.mjs';

const repoRoot = process.cwd();
const workspaceRoot = path.resolve(repoRoot, '..');

const deployResult = validateDeploy(repoRoot);
assert.equal(
  deployResult.ok,
  true,
  `deploy validation failed: ${(deployResult.errors ?? []).join('; ')}`,
);
assert.equal(deployResult.appId, 'sdkwork-deployments');
assert.equal(deployResult.runtimeCode, 'deploy');

const plan = planDeploy(repoRoot);
assert.equal(plan.ok, true, `deploy plan failed: ${(plan.errors ?? []).join('; ')}`);
assert.ok(plan.topology, 'plan must include topology for nginx render');
assert.ok(plan.overrides !== undefined, 'plan must include overrides for nginx render');
assert.equal(
  plan.upstreams.application,
  'http://127.0.0.1:3900',
  'deploy plan must resolve profile env bind to loopback upstream',
);

const deployAppSite = renderNginxSite(plan, 'deploy-app.sdkwork.com');
assert.match(deployAppSite.mainConfig, /# surface: application\.app-http/);
assert.match(deployAppSite.mainConfig, /location \/app\/v3\/api\//);
assert.doesNotMatch(deployAppSite.mainConfig, /location \/backend\/v3\/api\//);

const deployAdminSite = renderNginxSite(plan, 'deploy-admin.sdkwork.com');
assert.match(deployAdminSite.mainConfig, /# surface: application\.backend-http/);
assert.match(deployAdminSite.mainConfig, /location \/backend\/v3\/api\//);
assert.doesNotMatch(deployAdminSite.mainConfig, /location \/app\/v3\/api\//);

const deployPublicSite = renderNginxSite(plan, 'deploy.sdkwork.com');
assert.match(deployPublicSite.mainConfig, /# surface: application\.public-ingress/);
assert.match(deployPublicSite.mainConfig, /location \/app\/v3\/api\//);
assert.match(deployPublicSite.mainConfig, /location \/backend\/v3\/api\//);
assert.doesNotMatch(deployPublicSite.mainConfig, /127\.0\.0\.1:8080/);

const stripPlan = {
  ...plan,
  expose: plan.expose.map((item) =>
    item.domain === 'deploy-app.sdkwork.com'
      ? { ...item, apiPathStyle: 'strip-prefix' }
      : item,
  ),
};
const stripSite = renderNginxSite(stripPlan, 'deploy-app.sdkwork.com');
assert.match(
  stripSite.mainConfig,
  /location \/app\/v3\/api\/[\s\S]*proxy_pass http:\/\/127\.0\.0\.1:3900\//,
  'strip-prefix must proxy to upstream root',
);

const mailTopology = loadTopology(path.join(workspaceRoot, 'sdkwork-mail'));
assert.equal(
  resolveDomainSurfaceId(mailTopology, 'mail.sdkwork.com'),
  'application.public-ingress',
  'domain matching must be case-insensitive against topology hosts',
);

assert.deepEqual(
  validateDeploySchema({ version: 2, profile: 'x', expose: [] }),
  ['version: must be 1'],
  'schema validation must reject invalid version',
);

const nginxInspection = inspectNginxConfig(deployPublicSite.mainConfig);
assert.equal(nginxInspection.valid, true, nginxInspection.errors?.join('; '));

const imResult = validateDeploy(path.join(workspaceRoot, 'sdkwork-im'));
assert.equal(
  imResult.ok,
  true,
  `sdkwork-im deploy validation failed: ${(imResult.errors ?? []).join('; ')}`,
);

const mailPlan = planDeploy(path.join(workspaceRoot, 'sdkwork-mail'));
assert.equal(mailPlan.ok, true, 'sdkwork-mail deploy plan must succeed');
const mailSite = renderNginxSite(mailPlan, 'mail.sdkwork.com');
assert.match(
  mailSite.mainConfig,
  /map \$http_user_agent \$sdkwork_sdkwork_mail_surface \{/,
  'adaptive nginx must define UA map',
);
assert.match(
  mailSite.mainConfig,
  /include\s+\/etc\/nginx\/snippets\/sdkwork\/mail\.sdkwork\.com\.web\.\$sdkwork_sdkwork_mail_surface_final\.conf;/,
  'adaptive nginx must include variable snippet selector',
);
assert.doesNotMatch(
  mailSite.mainConfig,
  /set\s+\$root\s+/,
  'adaptive nginx must not use variable root SPA fallback',
);

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'sdkwork-deploy-contract-'));
try {
  const rendered = renderNginxSite(mailPlan, 'mail.sdkwork.com', { snippetDir: tmpDir });
  assert.ok(rendered.snippets.length >= 2, 'nginx render must emit pc and h5 web snippets');
} finally {
  fs.rmSync(tmpDir, { recursive: true, force: true });
}

process.stdout.write('deploy-standard.contract.test.mjs passed\n');
