import fs from "node:fs";
import path from "node:path";
import { parse as parseYaml } from "yaml";
import { migrateOpenApiDocument } from "../../sdkwork-specs/tools/lib/migrate-openapi-legacy-envelope.mjs";

const root = process.cwd();

const surfaces = [
  {
    yamlPath: "apis/app-api/deploy/openapi.yaml",
    jsonAuthorityPath: "apis/app-api/deploy/deploy-app-api.openapi.json",
    sdkJsonPath: "sdks/sdkwork-deploy-app-sdk/openapi/deploy-app-api.openapi.json",
    routeManifestPath:
      "sdks/_route-manifests/app-api/sdkwork-routes-deploy-app-api.route-manifest.json",
    crateDir: "crates/sdkwork-routes-deploy-app-api",
    manifestFn: "app_route_manifest",
    packageName: "sdkwork-routes-deploy-app-api",
    surface: "app-api",
    apiAuthority: "sdkwork-deploy-app-api",
    sdkFamily: "sdkwork-deploy-app-sdk",
    packageName: "@sdkwork/deploy-app-sdk",
    transportPackageName: "sdkwork-deploy-app-sdk-generated-typescript",
    prefix: "/app/v3/api",
    domainTag: "deploy",
  },
  {
    yamlPath: "apis/backend-api/deploy/openapi.yaml",
    jsonAuthorityPath: "apis/backend-api/deploy/deploy-backend-api.openapi.json",
    sdkJsonPath: "sdks/sdkwork-deploy-backend-sdk/openapi/deploy-backend-api.openapi.json",
    routeManifestPath:
      "sdks/_route-manifests/backend-api/sdkwork-routes-deploy-backend-api.route-manifest.json",
    crateDir: "crates/sdkwork-routes-deploy-backend-api",
    manifestFn: "backend_route_manifest",
    packageName: "sdkwork-routes-deploy-backend-api",
    surface: "backend-api",
    apiAuthority: "sdkwork-deploy-backend-api",
    sdkFamily: "sdkwork-deploy-backend-sdk",
    packageName: "@sdkwork/deploy-backend-sdk",
    transportPackageName: "sdkwork-deploy-backend-sdk-generated-typescript",
    prefix: "/backend/v3/api",
    domainTag: "deploy",
  },
];

function writeText(relativePath, content) {
  const target = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content.replace(/\r\n/g, "\n"), "utf8");
  console.log(`wrote ${relativePath}`);
}

function writeJson(relativePath, value) {
  writeText(relativePath, `${JSON.stringify(value, null, 2)}\n`);
}

function assertWellFormedUnicode(value, sourcePath, pathSegments = []) {
  if (typeof value === "string") {
    if (/[\uD800-\uDFFF]/.test(value)) {
      const pointer = pathSegments.length === 0 ? "/" : `/${pathSegments.join("/")}`;
      throw new Error(`${sourcePath}${pointer} contains an unpaired UTF-16 surrogate`);
    }
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((item, index) =>
      assertWellFormedUnicode(item, sourcePath, [...pathSegments, String(index)]),
    );
    return;
  }
  if (value && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) {
      assertWellFormedUnicode(item, sourcePath, [...pathSegments, key]);
    }
  }
}

function sdkManifest(profile) {
  const familyRoot = `sdks/${profile.sdkFamily}/`;
  if (!profile.sdkJsonPath.startsWith(familyRoot)) {
    throw new Error(`${profile.sdkJsonPath} is outside ${familyRoot}`);
  }
  const familyOpenApiPath = profile.sdkJsonPath.slice(familyRoot.length);
  return {
    schemaVersion: 1,
    apiAuthority: profile.apiAuthority,
    sdkOwner: "sdkwork-deploy",
    sdkDependencies: [],
    sdkFamily: profile.sdkFamily,
    sdkName: profile.sdkFamily,
    packageName: profile.packageName,
    transportPackageName: profile.transportPackageName,
    openApiPath: familyOpenApiPath,
    surface: profile.surface,
    authoritySpec: familyOpenApiPath,
  };
}

function enrichOpenApi(openapi, profile) {
  const enriched = structuredClone(openapi);
  for (const [pathKey, pathItem] of Object.entries(enriched.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      operation["x-sdkwork-api-surface"] = profile.surface;
      operation["x-sdkwork-owner"] = "sdkwork-deploy";
      operation["x-sdkwork-api-authority"] = profile.apiAuthority;
      operation["x-sdkwork-request-context"] = "WebRequestContext";
      operation["x-sdkwork-auth-mode"] =
        operation["x-sdkwork-auth-mode"] ?? "dual-token";
      if (!operation["x-sdkwork-permission"] && operation.operationId) {
        const [resource, action] = operation.operationId.split(".");
        const verb = action?.includes("list") || action?.includes("retrieve")
          ? "read"
          : "write";
        operation["x-sdkwork-permission"] = `deploy.${resource}.${verb}`;
      }
      if (
        method === "post" &&
        (operation.operationId?.includes("create") ||
          operation.operationId?.includes("rollback") ||
          operation.operationId?.includes("reload") ||
          operation.operationId?.includes("deploy") ||
          operation.operationId?.includes("verify"))
      ) {
        operation["x-sdkwork-idempotent"] = true;
      }
    }
  }
  return enriched;
}

function extractRoutes(openapi, profile) {
  const routes = [];
  for (const [pathKey, pathItem] of Object.entries(openapi.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      routes.push({
        method: method.toUpperCase(),
        path: pathKey,
        operationId: operation.operationId,
        tags: operation.tags ?? [profile.domainTag],
        auth: {
          mode: operation["x-sdkwork-auth-mode"] ?? "dual-token",
          required: true,
        },
        handler: { module: "crate::routes", name: null },
        ownership: {
          owner: "sdkwork-deploy",
          apiAuthority: profile.apiAuthority,
        },
        requestContext: "WebRequestContext",
        apiSurface: profile.surface,
        permission: operation["x-sdkwork-permission"] ?? null,
        idempotent: operation["x-sdkwork-idempotent"] === true,
      });
    }
  }
  return routes;
}

function httpRouteAuthHelper(authMode) {
  return authMode === "api-key" ? "api_key" : "dual_token";
}

function httpMethodRust(method) {
  return { GET: "Get", POST: "Post", PATCH: "Patch", PUT: "Put", DELETE: "Delete" }[
    method
  ];
}

function writeHttpRouteManifestRust(crateDir, fnName, routes) {
  const lines = [
    "// @generated by tools/materialize_deploy_phase1_contracts.mjs — do not edit",
    "",
    "use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};",
    "",
    "const HTTP_ROUTES: &[HttpRoute] = &[",
  ];
  for (const route of routes) {
    const auth = httpRouteAuthHelper(route.auth?.mode ?? "dual-token");
    const suffix = [
      route.permission ? `.with_required_permission("${route.permission}")` : "",
      route.idempotent ? ".with_idempotent(true)" : "",
    ].join("");
    lines.push(`    HttpRoute::${auth}(`);
    lines.push(`        HttpMethod::${httpMethodRust(route.method)},`);
    lines.push(`        "${route.path}",`);
    lines.push(`        "${route.tags[0] ?? "deploy"}",`);
    lines.push(`        "${route.operationId}",`);
    lines.push(`    )${suffix},`);
  }
  lines.push("];", "", `pub fn ${fnName}() -> HttpRouteManifest {`, "    HttpRouteManifest::new(HTTP_ROUTES)", "}", "");
  writeText(`${crateDir}/src/http_route_manifest.rs`, lines.join("\n"));
}

for (const profile of surfaces) {
  const yaml = parseYaml(fs.readFileSync(path.join(root, profile.yamlPath), "utf8"));
  assertWellFormedUnicode(yaml, profile.yamlPath);
  const openapi = migrateOpenApiDocument(enrichOpenApi(yaml, profile));
  writeJson(profile.jsonAuthorityPath, openapi);
  writeJson(profile.sdkJsonPath, openapi);
  const routes = extractRoutes(openapi, profile);
  writeJson(profile.routeManifestPath, {
    schemaVersion: 1,
    kind: "sdkwork.route.manifest",
    packageName: profile.packageName,
    surface: profile.surface,
    owner: "sdkwork-deploy",
    domain: "platform",
    capability: "deploy",
    apiAuthority: profile.apiAuthority,
    sdkFamily: profile.sdkFamily,
    prefix: profile.prefix,
    source: {
      crateRoot: profile.crateDir,
      crateImport: profile.packageName.replaceAll("-", "_"),
      openApiAuthority: profile.sdkJsonPath,
    },
    routes,
  });
  writeHttpRouteManifestRust(profile.crateDir, profile.manifestFn, routes);
}

writeJson("apis/authority-manifest.json", {
  schemaVersion: 1,
  kind: "sdkwork.api.authority.manifest",
  surfaces: surfaces.map((profile) => ({
    authorityPath: profile.jsonAuthorityPath,
    sdkPath: profile.sdkJsonPath,
  })),
});

for (const profile of surfaces) {
  writeJson(`sdks/${profile.sdkFamily}/sdk-manifest.json`, sdkManifest(profile));
}

console.log("deploy phase-1 contracts materialized");
