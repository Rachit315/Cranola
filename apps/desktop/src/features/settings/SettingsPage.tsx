import { useQuery } from '@tanstack/react-query';
import { FolderOpen } from 'lucide-react';
import { useState } from 'react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button';
import { ErrorNotice } from '../../components/ErrorNotice';
import { SelectField, TextField } from '../../components/Field';
import { commands } from '../../ipc/commands';
import type { Settings } from '../../ipc/generated/Settings';
import type { Theme } from '../../ipc/generated/Theme';
import { ApiKeys } from '../ai/ApiKeys';
import { OllamaStatusCard } from '../ai/OllamaStatusCard';
import { ProviderPicker } from '../ai/ProviderPicker';
import { MicrophoneCheck, SystemAudioCheck } from '../audio/AudioChecks';
import { DevicePicker } from '../audio/DevicePicker';
import { ProfileForm } from './ProfileForm';
import { useUpdateSettings } from './useSettings';

function Section({
  id,
  title,
  description,
  children,
}: {
  id: string;
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section aria-labelledby={id} className="border-t border-line py-8 first:border-t-0 first:pt-0">
      <h2 id={id} className="text-lg font-semibold text-ink">
        {title}
      </h2>
      {description && <p className="mt-1 text-sm text-ink-muted">{description}</p>}
      <div className="mt-5 flex flex-col gap-5">{children}</div>
    </section>
  );
}

function OllamaUrlField({ settings }: { settings: Settings }) {
  const { t } = useTranslation();
  const update = useUpdateSettings();
  const [url, setUrl] = useState(settings.ai.ollamaBaseUrl);
  return (
    <form
      className="flex flex-col gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        update.mutate({ section: 'ai', value: { ...settings.ai, ollamaBaseUrl: url.trim() } });
      }}
    >
      <div className="flex items-end gap-2">
        <div className="flex-1">
          <TextField
            label={t('ai.ollama.urlLabel')}
            value={url}
            spellCheck={false}
            onChange={(e) => {
              setUrl(e.target.value);
            }}
          />
        </div>
        <Button
          type="submit"
          busy={update.isPending}
          disabled={url.trim() === settings.ai.ollamaBaseUrl}
        >
          {t('common.save')}
        </Button>
      </div>
      {update.error && <ErrorNotice error={update.error} />}
    </form>
  );
}

function DataSection() {
  const { t } = useTranslation();
  const info = useQuery({ queryKey: ['app-info'], queryFn: commands.getAppInfo });
  return (
    <>
      <p className="text-sm text-ink-muted">{t('settings.data.privacy')}</p>
      {info.data && (
        <dl className="grid grid-cols-[auto_1fr_auto] items-center gap-x-4 gap-y-2 text-sm">
          <dt className="text-ink-faint">{t('settings.data.version')}</dt>
          <dd className="col-span-2 text-ink">{info.data.version}</dd>
          <dt className="text-ink-faint">{t('settings.data.dataFolder')}</dt>
          <dd className="font-mono text-xs break-all text-ink">{info.data.dataDir}</dd>
          <dd>
            <Button
              variant="ghost"
              icon={<FolderOpen className="size-4" aria-hidden />}
              onClick={() => {
                void commands.openLocation('dataFolder');
              }}
            >
              {t('common.open')}
            </Button>
          </dd>
          <dt className="text-ink-faint">{t('settings.data.logsFolder')}</dt>
          <dd className="font-mono text-xs break-all text-ink">{info.data.logDir}</dd>
          <dd>
            <Button
              variant="ghost"
              icon={<FolderOpen className="size-4" aria-hidden />}
              onClick={() => {
                void commands.openLocation('logsFolder');
              }}
            >
              {t('common.open')}
            </Button>
          </dd>
        </dl>
      )}
    </>
  );
}

export function SettingsPage({ settings }: { settings: Settings }) {
  const { t } = useTranslation();
  const update = useUpdateSettings();

  return (
    <div className="mx-auto w-full max-w-2xl px-8 py-10">
      <h1 className="mb-8 font-serif text-3xl text-ink">{t('settings.title')}</h1>

      <Section id="settings-general" title={t('settings.general.title')}>
        <div className="max-w-xs">
          <SelectField
            label={t('settings.general.theme')}
            value={settings.general.theme}
            options={(['system', 'light', 'dark'] as Theme[]).map((v) => ({
              value: v,
              label: t(`settings.general.themes.${v}`),
            }))}
            onChange={(e) => {
              update.mutate({
                section: 'general',
                value: { ...settings.general, theme: e.target.value as Theme },
              });
            }}
          />
        </div>
      </Section>

      <Section
        id="settings-profile"
        title={t('settings.profile.title')}
        description={t('settings.profile.description')}
      >
        <ProfileForm initial={settings.profile} submitLabel={t('common.save')} />
      </Section>

      <Section
        id="settings-audio"
        title={t('settings.audio.title')}
        description={t('settings.audio.description')}
      >
        <DevicePicker
          value={settings.audio}
          onChange={(audio) => {
            update.mutate({ section: 'audio', value: audio });
          }}
        />
        <MicrophoneCheck deviceId={settings.audio.inputDeviceId} />
        <SystemAudioCheck deviceId={settings.audio.outputDeviceId} />
      </Section>

      <Section
        id="settings-ai"
        title={t('settings.ai.title')}
        description={t('settings.ai.description')}
      >
        <ProviderPicker
          value={settings.ai.llmProvider}
          onChange={(llmProvider) => {
            update.mutate({ section: 'ai', value: { ...settings.ai, llmProvider } });
          }}
        />
        <OllamaUrlField key={settings.ai.ollamaBaseUrl} settings={settings} />
        <OllamaStatusCard baseUrl={settings.ai.ollamaBaseUrl} />
        <ApiKeys />
      </Section>

      <Section id="settings-data" title={t('settings.data.title')}>
        <DataSection />
      </Section>

      {update.error && <ErrorNotice error={update.error} />}
    </div>
  );
}
