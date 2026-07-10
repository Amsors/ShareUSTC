import request from './request';
import type { UploadResourceRequest, UploadResourceResponse } from '../types/resource';
import type { ImageUploadResponse } from '../types/image';

export interface OssStatusResponse {
  storageBackend: 'local' | 'oss';
  stsEnabled: boolean;
  signedUrlExpiry: number;
}

export interface OssStsTokenRequest {
  fileType: 'resource' | 'image';
  fileName: string;
  fileSize?: number;
  contentType?: string;
}

interface OssStsTokenResponseBase {
  uploadMode: 'sts' | 'signed_url';
  uploadKey: string;
  expiresIn: number;
  storageBackend: 'local' | 'oss';
}

export interface OssStsCredentialsResponse extends OssStsTokenResponseBase {
  uploadMode: 'sts';
  accessKeyId: string;
  accessKeySecret: string;
  securityToken: string;
  expiration: string;
  bucket: string;
  region: string;
  endpoint: string;
}

export interface OssSignedUrlUploadResponse extends OssStsTokenResponseBase {
  uploadMode: 'signed_url';
  uploadUrl: string;
}

export type OssStsTokenResponse = OssStsCredentialsResponse | OssSignedUrlUploadResponse;

export interface ResourceUploadCallbackRequest extends UploadResourceRequest {
  ossKey: string;
}

export interface ImageUploadCallbackRequest {
  ossKey: string;
  originalName?: string;
}

export const getOssStatus = async (): Promise<OssStatusResponse> => {
  return request({
    url: '/oss/status',
    method: 'get',
  }) as Promise<OssStatusResponse>;
};

export const getStsToken = async (data: OssStsTokenRequest): Promise<OssStsTokenResponse> => {
  return request({
    url: '/oss/sts-token',
    method: 'post',
    data,
  }) as Promise<OssStsTokenResponse>;
};

export const resourceUploadCallback = async (
  data: ResourceUploadCallbackRequest
): Promise<UploadResourceResponse> => {
  return request({
    url: '/oss/callback/resource',
    method: 'post',
    data,
  }) as Promise<UploadResourceResponse>;
};

export const imageUploadCallback = async (
  data: ImageUploadCallbackRequest
): Promise<ImageUploadResponse> => {
  return request({
    url: '/oss/callback/image',
    method: 'post',
    data,
  }) as Promise<ImageUploadResponse>;
};
