import { invoke } from '@tauri-apps/api/core';

/** Mirrors `sw_core::info::AppInfo`. */
export interface AppInfo {
  name: string;
  version: string;
  os: string;
  arch: string;
  dataDir: string;
}

/** Mirrors the serialized form of `sw_core::error::AppError`. */
export interface AppErrorPayload {
  code: string;
  message: string;
  detail: string | null;
  retryable: boolean;
}

export function isAppErrorPayload(value: unknown): value is AppErrorPayload {
  if (typeof value !== 'object' || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.code === 'string' && typeof v.message === 'string';
}

export function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>('get_app_info');
}
