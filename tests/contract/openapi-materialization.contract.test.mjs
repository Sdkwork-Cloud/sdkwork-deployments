#!/usr/bin/env node
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';

const repoRoot = process.cwd();
const surfaces = [
  {
    source: 'apis/app-api/deploy/openapi.yaml',
    authority: 'apis/app-api/deploy/deploy-app-api.openapi.json',
    sdk: 'sdks/sdkwork-deploy-app-sdk/openapi/deploy-app-api.openapi.json',
    sdkManifest: 'sdks/sdkwork-deploy-app-sdk/sdk-manifest.json',
    componentSpec: 'sdks/sdkwork-deploy-app-sdk/specs/component.spec.json',
    apiAuthority: 'sdkwork-deploy-app-api',
    apiSurface: 'app-api',
    apiPrefix: '/app/v3/api',
  },
  {
    source: 'apis/backend-api/deploy/openapi.yaml',
    authority: 'apis/backend-api/deploy/deploy-backend-api.openapi.json',
    sdk: 'sdks/sdkwork-deploy-backend-sdk/openapi/deploy-backend-api.openapi.json',
    sdkManifest: 'sdks/sdkwork-deploy-backend-sdk/sdk-manifest.json',
    componentSpec: 'sdks/sdkwork-deploy-backend-sdk/specs/component.spec.json',
    apiAuthority: 'sdkwork-deploy-backend-api',
    apiSurface: 'backend-api',
    apiPrefix: '/backend/v3/api',
  },
];

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), 'utf8'));
}

function assertWellFormedUnicode(value, sourcePath, pathSegments = []) {
  if (typeof value === 'string') {
    assert.doesNotMatch(
      value,
      /[\uD800-\uDFFF]/,
      `${sourcePath}/${pathSegments.join('/')} contains an unpaired UTF-16 surrogate`,
    );
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((item, index) =>
      assertWellFormedUnicode(item, sourcePath, [...pathSegments, String(index)]),
    );
    return;
  }
  if (value && typeof value === 'object') {
    for (const [key, item] of Object.entries(value)) {
      assertWellFormedUnicode(item, sourcePath, [...pathSegments, key]);
    }
  }
}

function collectDocumentation(value, pathSegments = [], result = new Map()) {
  if (Array.isArray(value)) {
    value.forEach((item, index) =>
      collectDocumentation(item, [...pathSegments, String(index)], result),
    );
    return result;
  }
  if (!value || typeof value !== 'object') {
    return result;
  }
  for (const [key, item] of Object.entries(value)) {
    const itemPath = [...pathSegments, key];
    if ((key === 'summary' || key === 'description') && typeof item === 'string') {
      result.set(itemPath.join('/'), item);
    }
    collectDocumentation(item, itemPath, result);
  }
  return result;
}

function assertMaterializedDocumentationMatchesSource(source, authority, sourcePath) {
  const sourceDocumentation = collectDocumentation(source);
  for (const [documentationPath, value] of collectDocumentation(authority)) {
    assert.equal(
      sourceDocumentation.get(documentationPath),
      value,
      `${sourcePath}/${documentationPath} must match its materialized authority`,
    );
  }
}

function assertOwnerOnlyOperations(authority, surface) {
  for (const [operationPath, pathItem] of Object.entries(authority.paths ?? {})) {
    assert.ok(
      operationPath.startsWith(surface.apiPrefix),
      `${surface.authority} contains an operation outside ${surface.apiPrefix}: ${operationPath}`,
    );
    for (const [method, operation] of Object.entries(pathItem)) {
      if (!['get', 'post', 'put', 'patch', 'delete'].includes(method)) {
        continue;
      }
      assert.equal(operation['x-sdkwork-owner'], 'sdkwork-deploy');
      assert.equal(operation['x-sdkwork-api-authority'], surface.apiAuthority);
      assert.equal(operation['x-sdkwork-api-surface'], surface.apiSurface);
    }
  }
}

for (const surface of surfaces) {
  const source = parseYaml(fs.readFileSync(path.join(repoRoot, surface.source), 'utf8'));
  const authority = readJson(surface.authority);
  const sdk = readJson(surface.sdk);
  const sdkManifest = readJson(surface.sdkManifest);
  const componentSpec = readJson(surface.componentSpec);

  assertWellFormedUnicode(source, surface.source);
  assertMaterializedDocumentationMatchesSource(source, authority, surface.source);
  assertOwnerOnlyOperations(authority, surface);
  assert.deepEqual(sdk, authority, `${surface.sdk} must match ${surface.authority}`);
  assert.equal(sdkManifest.sdkOwner, 'sdkwork-deploy');
  assert.equal(sdkManifest.apiAuthority, surface.apiAuthority);
  assert.deepEqual(
    sdkManifest.sdkDependencies,
    componentSpec.contracts.sdkDependencies,
    `${surface.sdkManifest} sdkDependencies must match ${surface.componentSpec}`,
  );
}

const appRouteManifest = readJson(
  'sdks/_route-manifests/app-api/sdkwork-routes-deploy-app-api.route-manifest.json',
);
const compositionRoute = appRouteManifest.routes.find(
  (route) => route.operationId === 'sites.composition.update',
);
assert.ok(compositionRoute, 'sites.composition.update must be materialized into the app route manifest');
assert.equal(compositionRoute.idempotent, true);
assert.equal(compositionRoute.permission, 'deploy.sites.write');
assert.deepEqual(compositionRoute.auth, { mode: 'dual-token', required: true });

process.stdout.write('openapi-materialization.contract.test.mjs passed\n');
