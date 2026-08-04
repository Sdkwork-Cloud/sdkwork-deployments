import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkAppConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { DomainApi, createDomainApi } from './api/domain';
import { SiteApi, createSiteApi } from './api/site';
import { DeploymentApi, createDeploymentApi } from './api/deployment';
import { ReleaseApi, createReleaseApi } from './api/release';
import { EnvVariableApi, createEnvVariableApi } from './api/env-variable';
import { CertificateApi, createCertificateApi } from './api/certificate';
import { UploadSessionApi, createUploadSessionApi } from './api/upload-session';
import { ArtifactApi, createArtifactApi } from './api/artifact';
import { MonitorApi, createMonitorApi } from './api/monitor';
import { AppApi, createAppApi } from './api/app';
import { BuildApi, createBuildApi } from './api/build';
import { PackageApi, createPackageApi } from './api/package';
import { SigningApi, createSigningApi } from './api/signing';

export class SdkworkDeployAppClient {
  private httpClient: HttpClient;

  public readonly domain: DomainApi;
  public readonly site: SiteApi;
  public readonly deployment: DeploymentApi;
  public readonly release: ReleaseApi;
  public readonly envVariable: EnvVariableApi;
  public readonly certificate: CertificateApi;
  public readonly uploadSession: UploadSessionApi;
  public readonly artifact: ArtifactApi;
  public readonly monitor: MonitorApi;
  public readonly app: AppApi;
  public readonly build: BuildApi;
  public readonly package: PackageApi;
  public readonly signing: SigningApi;

  constructor(config: SdkworkAppConfig) {
    this.httpClient = createHttpClient(config);
    this.domain = createDomainApi(this.httpClient);

    this.site = createSiteApi(this.httpClient);

    this.deployment = createDeploymentApi(this.httpClient);

    this.release = createReleaseApi(this.httpClient);

    this.envVariable = createEnvVariableApi(this.httpClient);

    this.certificate = createCertificateApi(this.httpClient);

    this.uploadSession = createUploadSessionApi(this.httpClient);

    this.artifact = createArtifactApi(this.httpClient);

    this.monitor = createMonitorApi(this.httpClient);

    this.app = createAppApi(this.httpClient);

    this.build = createBuildApi(this.httpClient);

    this.package = createPackageApi(this.httpClient);

    this.signing = createSigningApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkAppConfig): SdkworkDeployAppClient {
  return new SdkworkDeployAppClient(config);
}

export default SdkworkDeployAppClient;
