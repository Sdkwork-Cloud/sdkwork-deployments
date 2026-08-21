# Deployments

Deployment descriptors, Docker/Kubernetes examples, and release handoff documents for SDKWork Deploy.

- Canonical manifest: [deploy.yaml](deploy.yaml) (validated by `pnpm deploy:validate`)
- Docker: [docker/](docker/) — image, and the Compose templates
  `docker-compose.yml` / `docker-compose.external.yml` (container port **3900**)
- Kubernetes: [kubernetes/](kubernetes/)

This directory follows `../sdkwork-specs/DEPLOYMENT_SPEC.md` and `../sdkwork-specs/NGINX_SPEC.md`.
The Compose stack deploys `sdkwork-api-deployments-standalone-gateway` in `standalone`
deployment profile (external PostgreSQL/Redis).
