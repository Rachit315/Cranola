import { AlertTriangle, FolderOpen } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '../components/Button';
import { commands } from '../ipc/commands';
import type { StartupStatus } from '../ipc/generated/StartupStatus';

export function StartupErrorScreen({ status }: { status: StartupStatus }) {
  const { t } = useTranslation();
  const error = status.error;
  return (
    <main className="flex h-full items-center justify-center p-8">
      <section
        role="alert"
        className="w-full max-w-lg rounded-2xl border border-line bg-surface p-8 shadow-sm"
      >
        <AlertTriangle className="size-6 text-danger" aria-hidden />
        <h1 className="mt-4 font-serif text-2xl text-ink">{t('startup.title')}</h1>
        <p className="mt-2 text-sm text-ink-muted">{error?.message ?? t('startup.unknown')}</p>
        {error?.detail && (
          <p className="mt-3 rounded-lg bg-paper p-3 font-mono text-xs break-all text-ink-muted">
            {error.detail}
          </p>
        )}
        <p className="mt-4 text-sm text-ink-muted">{t('startup.help')}</p>
        {status.logDir && (
          <div className="mt-5">
            <Button
              icon={<FolderOpen className="size-4" aria-hidden />}
              onClick={() => {
                void commands.openLocation('logsFolder');
              }}
            >
              {t('errors.actions.openLogs')}
            </Button>
          </div>
        )}
      </section>
    </main>
  );
}
