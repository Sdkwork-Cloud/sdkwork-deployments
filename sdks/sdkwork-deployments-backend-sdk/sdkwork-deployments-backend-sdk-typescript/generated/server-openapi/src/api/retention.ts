import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { RetentionRunRequest, RetentionRunResponse } from '../types';


export class RetentionApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Apply platform retention policies */
  async run(body: RetentionRunRequest, requestOptions?: ApiRequestOptions): Promise<RetentionRunResponse> {
    return this.client.request<RetentionRunResponse>(backendApiPath(`/retention/run`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export function createRetentionApi(client: HttpClient): RetentionApi {
  return new RetentionApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
