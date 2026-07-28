# SDKWork Deploy Documentation

## Audience Routing

| Audience | Read first | Then read |
| --- | --- | --- |
| Product or business | [product/prd/PRD.md](product/prd/PRD.md) | [product/requirements/](product/requirements/) |
| Architect | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) | [architecture/decisions/](architecture/decisions/) |
| Developer | [guides/developer/README.md](guides/developer/README.md) | [engineering/plans/](engineering/plans/) |
| Operator | [guides/operator/README.md](guides/operator/README.md) | [runbooks/](runbooks/) |
| Integrator | [guides/integrator/README.md](guides/integrator/README.md) | repository `apis/` and `sdks/` |
| Agent | [../AGENTS.md](../AGENTS.md) | [standards-alignment.md](standards-alignment.md), [INDEX.yaml](INDEX.yaml) |

## Canon Documents

| Document | Path |
| --- | --- |
| Product PRD | [product/prd/PRD.md](product/prd/PRD.md) |
| Technical architecture | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) |
| Cloud publishing PRD | [product/prd/PRD-cloud-site-publishing-platform.md](product/prd/PRD-cloud-site-publishing-platform.md) |
| Cloud publishing architecture | [architecture/tech/TECH-cloud-site-publishing-control-plane.md](architecture/tech/TECH-cloud-site-publishing-control-plane.md) |
| Managed domain/TLS decision | [architecture/decisions/ADR-20260723-managed-domain-tls-control-plane.md](architecture/decisions/ADR-20260723-managed-domain-tls-control-plane.md) |
| Managed domain/TLS plan | [engineering/plans/PLAN-2026-0002-managed-domain-tls-control-plane.md](engineering/plans/PLAN-2026-0002-managed-domain-tls-control-plane.md) |
| Prelaunch control-plane convergence | [migrations/MIG-2026-0001-cloud-site-control-plane-convergence.md](migrations/MIG-2026-0001-cloud-site-control-plane-convergence.md) |
| Deployments SDK family rename | [migrations/MIG-2026-0002-deployments-sdk-family-rename.md](migrations/MIG-2026-0002-deployments-sdk-family-rename.md) |
| Commercial readiness review | [engineering/reviews/REVIEW-20260721-cloud-site-publishing-commercial-readiness.md](engineering/reviews/REVIEW-20260721-cloud-site-publishing-commercial-readiness.md) |

## Related Specs

- `DOCUMENTATION_SPEC.md`
- `SDKWORK_WORKSPACE_SPEC.md`
- `REQUIREMENTS_SPEC.md`
- `ARCHITECTURE_DECISION_SPEC.md`

## Verification

```bash
node ../sdkwork-specs/tools/check-repository-docs-standard.mjs --root .
```
