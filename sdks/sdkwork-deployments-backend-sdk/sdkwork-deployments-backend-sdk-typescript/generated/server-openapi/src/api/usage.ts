import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { IngestUsageEventsRequest, UsageIngestResult, UsageReconciliationRequest, UsageReconciliationResponse } from '../types';


export class UsageReconcileApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Rebuild the daily usage aggregate from retained usage facts */
  async create(body: UsageReconciliationRequest, requestOptions?: ApiRequestOptions): Promise<UsageReconciliationResponse> {
    return this.client.request<UsageReconciliationResponse>(backendApiPath(`/usage/reconcile`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class UsageIngestApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Ingest Web Server traffic usage events (per-domain / per-server-IP) */
  async create(body: IngestUsageEventsRequest, requestOptions?: ApiRequestOptions): Promise<UsageIngestResult> {
    return this.client.request<UsageIngestResult>(backendApiPath(`/usage/ingest`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class UsageApi {
  public readonly ingest: UsageIngestApi;
  public readonly reconcile: UsageReconcileApi;

  constructor(client: HttpClient) {
    this.ingest = new UsageIngestApi(client);
    this.reconcile = new UsageReconcileApi(client);
  }

}

export function createUsageApi(client: HttpClient): UsageApi {
  return new UsageApi(client);
}
