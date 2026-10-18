import { useMutation } from '@tanstack/react-query';
import { CheckCircle2, Mic, Volume2, XCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button';
import { ErrorNotice } from '../../components/ErrorNotice';
import { commands } from '../../ipc/commands';
import type { AppError } from '../../ipc/errors';
import type { LoopbackCheck } from '../../ipc/generated/LoopbackCheck';
import type { MicCheck } from '../../ipc/generated/MicCheck';
import { levelPercent } from './queries';

function Result({ ok, children }: { ok: boolean; children: React.ReactNode }) {
  const Icon = ok ? CheckCircle2 : XCircle;
  return (
    <p className="flex items-start gap-2 text-sm text-ink" role="status">
      <Icon
        className={ok ? 'mt-0.5 size-4 text-accent' : 'mt-0.5 size-4 text-danger'}
        aria-hidden
      />
      <span>{children}</span>
    </p>
  );
}

interface CheckProps {
  deviceId: string | null;
  onResult?: ((ok: boolean) => void) | undefined;
}

export function MicrophoneCheck({ deviceId, onResult }: CheckProps) {
  const { t } = useTranslation();
  const check = useMutation<MicCheck, AppError>({
    mutationFn: () => commands.checkMicrophone(deviceId),
    onSuccess: (r) => onResult?.(r.heardSignal),
    onError: () => onResult?.(false),
  });
  const run = () => {
    check.mutate();
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <Button icon={<Mic className="size-4" aria-hidden />} busy={check.isPending} onClick={run}>
          {check.isPending ? t('audio.mic.listening') : t('audio.mic.test')}
        </Button>
        {check.isPending && (
          <span className="text-sm text-ink-muted">{t('audio.mic.speakNow')}</span>
        )}
      </div>
      {check.data && (
        <div className="flex flex-col gap-2">
          <div
            className="h-2 w-full overflow-hidden rounded-full bg-line"
            role="meter"
            aria-label={t('audio.mic.levelLabel')}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={levelPercent(check.data.peakDbfs)}
          >
            <div
              className="h-full rounded-full bg-accent transition-[width]"
              style={{ width: `${String(levelPercent(check.data.peakDbfs))}%` }}
            />
          </div>
          {check.data.heardSignal ? (
            <Result ok>{t('audio.mic.heard', { device: check.data.deviceName })}</Result>
          ) : (
            <Result ok={false}>
              {check.data.allZero ? t('audio.mic.allZero') : t('audio.mic.silent')}
            </Result>
          )}
        </div>
      )}
      {check.error && <ErrorNotice error={check.error} onRetry={run} />}
    </div>
  );
}

export function SystemAudioCheck({ deviceId, onResult }: CheckProps) {
  const { t } = useTranslation();
  const check = useMutation<LoopbackCheck, AppError>({
    mutationFn: () => commands.checkSystemAudio(deviceId),
    onSuccess: (r) => onResult?.(r.toneDetected),
    onError: () => onResult?.(false),
  });
  const run = () => {
    check.mutate();
  };

  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button
          icon={<Volume2 className="size-4" aria-hidden />}
          busy={check.isPending}
          onClick={run}
        >
          {check.isPending ? t('audio.system.playing') : t('audio.system.test')}
        </Button>
      </div>
      {check.data &&
        (check.data.toneDetected ? (
          <Result ok>{t('audio.system.captured', { device: check.data.deviceName })}</Result>
        ) : (
          <Result ok={false}>
            {t('audio.system.notCaptured', { device: check.data.deviceName })}
          </Result>
        ))}
      {check.error && <ErrorNotice error={check.error} onRetry={run} />}
    </div>
  );
}
