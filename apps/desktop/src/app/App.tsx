import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Route, Routes } from 'react-router';

import { ErrorNotice } from '../components/ErrorNotice';
import { commands } from '../ipc/commands';
import type { AppError } from '../ipc/errors';
import type { StartupStatus } from '../ipc/generated/StartupStatus';
import { HomePage } from '../features/home/HomePage';
import { Onboarding } from '../features/onboarding/Onboarding';
import { SettingsPage } from '../features/settings/SettingsPage';
import { useSettings } from '../features/settings/useSettings';
import { Shell } from './Shell';
import { StartupErrorScreen } from './StartupErrorScreen';
import { useApplyTheme } from './theme';

function Splash() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full items-center justify-center" role="status">
      <span className="text-sm text-ink-faint">{t('common.loading')}</span>
    </div>
  );
}

function ReadyApp() {
  const settings = useSettings();
  useApplyTheme(settings.data?.general.theme);

  if (settings.error) {
    return (
      <div className="mx-auto max-w-lg p-8">
        <ErrorNotice
          error={settings.error}
          onRetry={() => {
            void settings.refetch();
          }}
        />
      </div>
    );
  }
  if (!settings.data) return <Splash />;
  if (settings.data.onboarding.completedAt === null) {
    return <Onboarding settings={settings.data} />;
  }
  return (
    <Routes>
      <Route element={<Shell />}>
        <Route index element={<HomePage settings={settings.data} />} />
        <Route path="settings" element={<SettingsPage settings={settings.data} />} />
      </Route>
    </Routes>
  );
}

export function App() {
  const startup = useQuery<StartupStatus, AppError>({
    queryKey: ['startup'],
    queryFn: commands.getStartupStatus,
  });

  if (startup.error) {
    return (
      <StartupErrorScreen
        status={{
          ready: false,
          error: { code: startup.error.code, message: startup.error.message, detail: null },
          logDir: null,
        }}
      />
    );
  }
  if (!startup.data) return <Splash />;
  if (!startup.data.ready) return <StartupErrorScreen status={startup.data} />;
  return <ReadyApp />;
}
