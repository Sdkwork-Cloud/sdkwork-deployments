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
    return this.client.request<UsageReconciliationResponse>(backendApiPath(`/usage/reconcile`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export function createUsageApi(client: HttpClient): UsageApi {
  return new UsageApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
