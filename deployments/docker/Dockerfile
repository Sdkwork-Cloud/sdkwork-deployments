FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-deployments-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-deployments-standalone-gateway /usr/local/bin/sdkwork-deployments-standalone-gateway
COPY --from=builder /src/database /app/database
ENV SDKWORK_DEPLOY_APP_ROOT=/app
ENV SDKWORK_DEPLOY_ENVIRONMENT=production
ENV SDKWORK_DEPLOY_CONFIG_PROFILE=production
ENV SDKWORK_DEPLOY_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["sdkwork-deployments-standalone-gateway"]
