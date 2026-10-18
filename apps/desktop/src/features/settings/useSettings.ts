import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { commands } from '../../ipc/commands';
import type { AppError } from '../../ipc/errors';
import type { Settings } from '../../ipc/generated/Settings';
import type { SettingsUpdate } from '../../ipc/generated/SettingsUpdate';

export const settingsKey = ['settings'] as const;

export function useSettings() {
  return useQuery<Settings, AppError>({ queryKey: settingsKey, queryFn: commands.getSettings });
}

export function useUpdateSettings() {
  const client = useQueryClient();
  return useMutation<Settings, AppError, SettingsUpdate>({
    mutationFn: commands.updateSettings,
    onSuccess: (settings) => {
      client.setQueryData(settingsKey, settings);
    },
  });
}

export function useCompleteOnboarding() {
  const client = useQueryClient();
  return useMutation<Settings, AppError>({
    mutationFn: commands.completeOnboarding,
    onSuccess: (settings) => {
      client.setQueryData(settingsKey, settings);
    },
  });
}
