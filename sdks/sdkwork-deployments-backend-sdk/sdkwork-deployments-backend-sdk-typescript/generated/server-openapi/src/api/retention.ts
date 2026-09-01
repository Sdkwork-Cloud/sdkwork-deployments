import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { RetentionRunRequest, RetentionRunResponse } from '../types';


export class RetentionRunApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Apply platform retention policies */
  async create(body: RetentionRunRequest, requestOptions?: ApiRequestOptions): Promise<RetentionRunResponse> {
    return this.client.request<RetentionRunResponse>(backendApiPath(`/retention/run`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class RetentionApi {
  public readonly run: RetentionRunApi;

  constructor(client: HttpClient) {
    this.run = new RetentionRunApi(client);
  }

}

export function createRetentionApi(client: HttpClient): RetentionApi {
  return new RetentionApi(client);
}
