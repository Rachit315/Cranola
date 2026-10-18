import { AlertCircle } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { commands } from '../ipc/commands';
import type { AppErrorPayload } from '../ipc/errors';
import { Button } from './Button';

/** Known error codes have translated copy; anything else shows Rust's message. */
function useErrorText(error: AppErrorPayload): string {
  const { t, i18n } = useTranslation();
  if (error.code === 'input.invalid') return error.message;
  const key = `errors.codes.${error.code}`;
  // Codes come from Rust at runtime, so the key can't be checked statically;
  // `exists` guards the lookup and the cast only satisfies the typed `t`.
  return i18n.exists(key) ? t(key as 'errors.codes.internal') : error.message;
}

interface ErrorNoticeProps {
  error: AppErrorPayload;
  onRetry?: (() => void) | undefined;
}

export function ErrorNotice({ error, onRetry }: ErrorNoticeProps) {
  const { t } = useTranslation();
  const text = useErrorText(error);
  const [showDetail, setShowDetail] = useState(false);

  const runAction = () => {
    if (error.action === 'open_settings:microphone') {
      void commands.openLocation('microphoneSettings');
    } else if (error.action === 'open_logs') {
      void commands.openLocation('logsFolder');
    }
  };

  return (
    <div role="alert" className="rounded-lg border border-danger/30 bg-danger/5 p-3 text-sm">
      <div className="flex items-start gap-2">
        <AlertCircle className="mt-0.5 size-4 shrink-0 text-danger" aria-hidden />
        <div className="flex-1">
          <p className="text-ink">{text}</p>
          {showDetail && error.detail && (
            <p className="mt-1 font-mono text-xs break-all text-ink-muted">{error.detail}</p>
          )}
          <div className="mt-2 flex flex-wrap gap-2">
            {error.action === 'open_settings:microphone' && (
              <Button onClick={runAction}>{t('errors.actions.openMicrophoneSettings')}</Button>
            )}
            {error.action === 'open_logs' && (
              <Button onClick={runAction}>{t('errors.actions.openLogs')}</Button>
            )}
            {onRetry && error.retryable && <Button onClick={onRetry}>{t('common.retry')}</Button>}
            {error.detail && !showDetail && (
              <Button
                variant="ghost"
                onClick={() => {
                  setShowDetail(true);
                }}
              >
                {t('errors.showDetails')}
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
