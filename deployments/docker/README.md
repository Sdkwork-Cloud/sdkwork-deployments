# SDKWork Deploy Docker Image

Build from repository root:

```bash
docker build -f deployments/docker/Dockerfile -t sdkwork-deploy-api-server:latest .
```

Run with SQLite (development only):

```bash
docker run --rm -p 3900:8080 \
  -e SDKWORK_DEPLOY_DATABASE_ENGINE=sqlite \
  -e SDKWORK_DEPLOY_DATABASE_URL=sqlite:///app/data/deploy.db \
  -e SDKWORK_DEPLOY_DATABASE_AUTO_MIGRATE=true \
  sdkwork-deploy-api-server:latest
```

Production deployments must use PostgreSQL and IAM database credentials via secrets.
