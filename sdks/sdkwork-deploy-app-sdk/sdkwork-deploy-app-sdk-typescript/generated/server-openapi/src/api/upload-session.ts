import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CompleteDeployUploadSessionRequest, CreateDeployUploadSessionRequest, DeployUploadSessionResponse } from '../types';


export class UploadSessionApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** 创建 Drive-backed 包上传会话 */
  async create(body: CreateDeployUploadSessionRequest, requestOptions?: ApiRequestOptions): Promise<DeployUploadSessionResponse> {
    return this.client.request<DeployUploadSessionResponse>(appApiPath(`/upload_sessions`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json' });
  }

/** 获取上传会话 */
  async retrieve(uploadSessionId: string, requestOptions?: ApiRequestOptions): Promise<DeployUploadSessionResponse> {
    return this.client.request<DeployUploadSessionResponse>(appApiPath(`/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any });
  }

/** 完成上传会话 */
  async complete(uploadSessionId: string, body: CompleteDeployUploadSessionRequest, requestOptions?: ApiRequestOptions): Promise<DeployUploadSessionResponse> {
    return this.client.request<DeployUploadSessionResponse>(appApiPath(`/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}/complete`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json' });
  }

/** 取消上传会话 */
  async cancel(uploadSessionId: string, requestOptions?: ApiRequestOptions): Promise<DeployUploadSessionResponse> {
    return this.client.request<DeployUploadSessionResponse>(appApiPath(`/upload_sessions/${serializePathParameter(uploadSessionId, { name: 'uploadSessionId', style: 'simple', explode: false })}/cancel`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any });
  }
}

export function createUploadSessionApi(client: HttpClient): UploadSessionApi {
  return new UploadSessionApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
