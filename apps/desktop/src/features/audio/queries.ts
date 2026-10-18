import { useQuery } from '@tanstack/react-query';

import { commands } from '../../ipc/commands';
import type { AppError } from '../../ipc/errors';
import type { AudioDevice } from '../../ipc/generated/AudioDevice';

export function useAudioDevices() {
  return useQuery<AudioDevice[], AppError>({
    queryKey: ['audio-devices'],
    queryFn: commands.listAudioDevices,
  });
}

/** Maps dBFS (-60..0) onto 0..100 for a level bar. */
export function levelPercent(dbfs: number): number {
  return Math.round(Math.min(100, Math.max(0, ((dbfs + 60) / 60) * 100)));
}
