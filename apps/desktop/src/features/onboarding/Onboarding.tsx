import { useQuery } from '@tanstack/react-query';
import { Headphones, Lock, ShieldCheck, Sparkles, Users } from 'lucide-react';
import { useState } from 'react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { Logo } from '../../components/Logo';
import { Button } from '../../components/Button';
import { ErrorNotice } from '../../components/ErrorNotice';
import { commands } from '../../ipc/commands';
import { AppError } from '../../ipc/errors';
import type { Settings } from '../../ipc/generated/Settings';
import { ApiKeys } from '../ai/ApiKeys';
import { OllamaStatusCard } from '../ai/OllamaStatusCard';
import { ProviderPicker } from '../ai/ProviderPicker';
import { MicrophoneCheck, SystemAudioCheck } from '../audio/AudioChecks';
import { ProfileForm } from '../settings/ProfileForm';
import { useCompleteOnboarding, useUpdateSettings } from '../settings/useSettings';
import { STEPS } from './steps';
import type { Step } from './steps';

interface StepFrameProps {
  title: string;
  body?: string;
  children?: ReactNode;
  footer: ReactNode;
}

function StepFrame({ title, body, children, footer }: StepFrameProps) {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="font-serif text-3xl tracking-tight text-ink">{title}</h1>
        {body && <p className="mt-2 text-sm leading-relaxed text-ink-muted">{body}</p>}
      </div>
      {children}
      <div className="flex items-center justify-between gap-3 border-t border-line pt-5">
        {footer}
      </div>
    </div>
  );
}

function Point({ icon, title, body }: { icon: ReactNode; title: string; body: string }) {
  return (
    <li className="flex gap-3">
      <span className="mt-0.5 text-accent">{icon}</span>
      <span>
        <span className="block text-sm font-medium text-ink">{title}</span>
        <span className="block text-sm text-ink-muted">{body}</span>
      </span>
    </li>
  );
}

const PERMISSION_DENIED = new AppError({
  code: 'audio.permission_denied',
  message: 'Microphone access is blocked by system privacy settings',
  detail: null,
  retryable: false,
  action: 'open_settings:microphone',
});

function MicrophonePermission() {
  const { t } = useTranslation();
  const permission = useQuery({
    queryKey: ['mic-permission'],
    queryFn: commands.getMicrophonePermission,
  });
  if (permission.data !== 'deniedSystemWide' && permission.data !== 'deniedForDesktopApps') {
    return null;
  }
  return (
    <div className="flex flex-col gap-2">
      <ErrorNotice error={PERMISSION_DENIED} />
      <p className="text-sm text-ink-muted">
        {permission.data === 'deniedSystemWide'
          ? t('onboarding.microphone.deniedSystemWide')
          : t('onboarding.microphone.deniedDesktopApps')}
      </p>
      <div>
        <Button
          onClick={() => {
            void permission.refetch();
          }}
        >
          {t('onboarding.microphone.recheck')}
        </Button>
      </div>
    </div>
  );
}

export function Onboarding({ settings }: { settings: Settings }) {
  const { t } = useTranslation();
  const [index, setIndex] = useState(0);
  const [micOk, setMicOk] = useState<boolean | null>(null);
  const [systemOk, setSystemOk] = useState<boolean | null>(null);
  const update = useUpdateSettings();
  const complete = useCompleteOnboarding();

  const step: Step = STEPS[index] ?? 'welcome';
  const next = () => {
    setIndex((i) => Math.min(i + 1, STEPS.length - 1));
  };
  const back = () => {
    setIndex((i) => Math.max(i - 1, 0));
  };
  const backButton =
    index > 0 ? (
      <Button variant="ghost" onClick={back}>
        {t('common.back')}
      </Button>
    ) : (
      <span />
    );

  let content: ReactNode;
  switch (step) {
    case 'welcome':
      content = (
        <StepFrame
          title={t('onboarding.welcome.title')}
          body={t('onboarding.welcome.body')}
          footer={
            <>
              <span />
              <Button variant="primary" onClick={next}>
                {t('onboarding.welcome.start')}
              </Button>
            </>
          }
        />
      );
      break;
    case 'privacy':
      content = (
        <StepFrame
          title={t('onboarding.privacy.title')}
          footer={
            <>
              {backButton}
              <Button variant="primary" onClick={next}>
                {t('common.continue')}
              </Button>
            </>
          }
        >
          <ul className="flex flex-col gap-4">
            <Point
              icon={<Lock className="size-4" aria-hidden />}
              title={t('onboarding.privacy.localTitle')}
              body={t('onboarding.privacy.localBody')}
            />
            <Point
              icon={<ShieldCheck className="size-4" aria-hidden />}
              title={t('onboarding.privacy.noAccountTitle')}
              body={t('onboarding.privacy.noAccountBody')}
            />
            <Point
              icon={<Sparkles className="size-4" aria-hidden />}
              title={t('onboarding.privacy.aiTitle')}
              body={t('onboarding.privacy.aiBody')}
            />
            <Point
              icon={<Users className="size-4" aria-hidden />}
              title={t('onboarding.privacy.consentTitle')}
              body={t('onboarding.privacy.consentBody')}
            />
          </ul>
        </StepFrame>
      );
      break;
    case 'microphone':
      content = (
        <StepFrame
          title={t('onboarding.microphone.title')}
          body={t('onboarding.microphone.body')}
          footer={
            <>
              {backButton}
              <Button variant={micOk ? 'primary' : 'secondary'} onClick={next}>
                {micOk ? t('common.continue') : t('common.skipForNow')}
              </Button>
            </>
          }
        >
          <MicrophonePermission />
          <MicrophoneCheck deviceId={settings.audio.inputDeviceId} onResult={setMicOk} />
        </StepFrame>
      );
      break;
    case 'systemAudio':
      content = (
        <StepFrame
          title={t('onboarding.systemAudio.title')}
          body={t('onboarding.systemAudio.body')}
          footer={
            <>
              {backButton}
              <Button variant={systemOk ? 'primary' : 'secondary'} onClick={next}>
                {systemOk ? t('common.continue') : t('common.skipForNow')}
              </Button>
            </>
          }
        >
          <SystemAudioCheck deviceId={settings.audio.outputDeviceId} onResult={setSystemOk} />
          <p className="flex items-start gap-2 text-sm text-ink-muted">
            <Headphones className="mt-0.5 size-4 shrink-0" aria-hidden />
            {t('onboarding.systemAudio.headphones')}
          </p>
        </StepFrame>
      );
      break;
    case 'ai':
      content = (
        <StepFrame
          title={t('onboarding.ai.title')}
          body={t('onboarding.ai.body')}
          footer={
            <>
              {backButton}
              <Button variant={settings.ai.llmProvider ? 'primary' : 'secondary'} onClick={next}>
                {settings.ai.llmProvider ? t('common.continue') : t('common.skipForNow')}
              </Button>
            </>
          }
        >
          <ProviderPicker
            value={settings.ai.llmProvider}
            onChange={(llmProvider) => {
              update.mutate({ section: 'ai', value: { ...settings.ai, llmProvider } });
            }}
          />
          {update.error && <ErrorNotice error={update.error} />}
          {settings.ai.llmProvider === 'ollama' && (
            <OllamaStatusCard baseUrl={settings.ai.ollamaBaseUrl} />
          )}
          {settings.ai.llmProvider !== null && settings.ai.llmProvider !== 'ollama' && <ApiKeys />}
        </StepFrame>
      );
      break;
    case 'profile':
      content = (
        <div className="flex flex-col gap-6">
          <div>
            <h1 className="font-serif text-3xl tracking-tight text-ink">
              {t('onboarding.profile.title')}
            </h1>
            <p className="mt-2 text-sm text-ink-muted">{t('onboarding.profile.body')}</p>
          </div>
          <ProfileForm
            initial={settings.profile}
            submitLabel={t('common.continue')}
            onSaved={next}
            secondaryAction={
              <>
                <Button variant="ghost" onClick={next}>
                  {t('common.skipForNow')}
                </Button>
                <Button variant="ghost" onClick={back}>
                  {t('common.back')}
                </Button>
              </>
            }
          />
        </div>
      );
      break;
    case 'done':
      content = (
        <StepFrame
          title={t('onboarding.done.title')}
          body={t('onboarding.done.body')}
          footer={
            <>
              {backButton}
              <Button
                variant="primary"
                busy={complete.isPending}
                onClick={() => {
                  complete.mutate();
                }}
              >
                {t('onboarding.done.finish')}
              </Button>
            </>
          }
        >
          {complete.error && <ErrorNotice error={complete.error} />}
        </StepFrame>
      );
      break;
  }

  return (
    <div className="flex min-h-full flex-col items-center bg-paper px-6 py-10">
      <div className="flex w-full max-w-xl items-center justify-between">
        <Logo className="size-8" />
        <p className="text-xs text-ink-faint" aria-live="polite">
          {t('onboarding.progress', { current: index + 1, total: STEPS.length })}
        </p>
      </div>
      <div
        className="mt-3 flex w-full max-w-xl gap-1.5"
        role="progressbar"
        aria-label={t('onboarding.progressLabel')}
        aria-valuemin={1}
        aria-valuemax={STEPS.length}
        aria-valuenow={index + 1}
      >
        {STEPS.map((s, i) => (
          <span
            key={s}
            className={
              i <= index ? 'h-1 flex-1 rounded-full bg-accent' : 'h-1 flex-1 rounded-full bg-line'
            }
          />
        ))}
      </div>
      <main className="mt-8 w-full max-w-xl rounded-2xl border border-line bg-surface p-8 shadow-sm">
        {content}
      </main>
    </div>
  );
}
