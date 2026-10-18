import { useQuery } from '@tanstack/react-query';

import { commands } from '../../ipc/commands';
import type { AppError } from '../../ipc/errors';
import type { OllamaStatus } from '../../ipc/generated/OllamaStatus';
import type { SecretStatus } from '../../ipc/generated/SecretStatus';

export const secretsKey = ['secrets'] as const;

export function useSecretStatus() {
  return useQuery<SecretStatus[], AppError>({
    queryKey: secretsKey,
    queryFn: commands.listSecretStatus,
  });
}

export function useOllamaStatus(baseUrl: string) {
  return useQuery<OllamaStatus, AppError>({
    queryKey: ['ollama', baseUrl],
    queryFn: () => commands.detectOllama(baseUrl),
  });
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '';
  const gb = bytes / 1e9;
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${Math.round(bytes / 1e6).toString()} MB`;
}
