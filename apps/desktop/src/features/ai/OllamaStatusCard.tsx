import { CheckCircle2, CircleDashed, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button';
import { formatBytes, useOllamaStatus } from './queries';

export function OllamaStatusCard({ baseUrl }: { baseUrl: string }) {
  const { t } = useTranslation();
  const status = useOllamaStatus(baseUrl);

  return (
    <div className="rounded-lg border border-line bg-surface p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-2">
          {status.data ? (
            <CheckCircle2 className="mt-0.5 size-4 text-accent" aria-hidden />
          ) : (
            <CircleDashed className="mt-0.5 size-4 text-ink-faint" aria-hidden />
          )}
          <div role="status">
            <p className="text-sm font-medium text-ink">
              {status.isPending
                ? t('ai.ollama.checking')
                : status.data
                  ? t('ai.ollama.running', { version: status.data.version })
                  : t('ai.ollama.notRunning')}
            </p>
            {status.error && (
              <p className="mt-1 text-sm text-ink-muted">
                {t('ai.ollama.notRunningHint', { url: baseUrl })}
              </p>
            )}
            {status.data?.models.length === 0 && (
              <p className="mt-1 text-sm text-ink-muted">{t('ai.ollama.noModels')}</p>
            )}
          </div>
        </div>
        <Button
          variant="ghost"
          icon={<RefreshCw className="size-4" aria-hidden />}
          busy={status.isFetching}
          onClick={() => {
            void status.refetch();
          }}
        >
          {t('ai.ollama.recheck')}
        </Button>
      </div>
      {status.data && status.data.models.length > 0 && (
        <ul className="mt-3 flex flex-col gap-1 border-t border-line pt-3 text-sm">
          {status.data.models.map((m) => (
            <li key={m.name} className="flex justify-between gap-4">
              <span className="font-mono text-ink">{m.name}</span>
              <span className="text-ink-faint">
                {[m.parameterSize, formatBytes(m.sizeBytes)].filter(Boolean).join(' · ')}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
