import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { UsageReconciliationRequest, UsageReconciliationResponse } from '../types';


export class UsageApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Rebuild the daily usage aggregate from retained usage facts */
  async reconcileDaily(body: UsageReconciliationRequest, requestOptions?: ApiRequestOptions): Promise<UsageReconciliationResponse> {
    return this.client.request<UsageReconciliationResponse>(backendApiPath(`/usage/reconcile`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export function createUsageApi(client: HttpClient): UsageApi {
  return new UsageApi(client);
}
