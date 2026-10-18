import { useEffect, useState } from 'react';

import { getAppInfo, isAppErrorPayload } from '../ipc/appInfo';
import type { AppInfo } from '../ipc/appInfo';

type State =
  { status: 'loading' } | { status: 'ready'; info: AppInfo } | { status: 'error'; message: string };

export function App() {
  const [state, setState] = useState<State>({ status: 'loading' });

  useEffect(() => {
    let cancelled = false;
    getAppInfo()
      .then((info) => {
        if (!cancelled) setState({ status: 'ready', info });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        const message = isAppErrorPayload(err)
          ? err.message
          : err instanceof Error
            ? err.message
            : String(err);
        setState({ status: 'error', message });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="flex h-full items-center justify-center p-8">
      <section className="w-full max-w-md rounded-xl border border-line bg-surface p-8 shadow-sm">
        <h1 className="font-serif text-3xl tracking-tight text-ink">Scribewell</h1>
        <p className="mt-2 text-sm text-ink-muted">
          Local-first meeting notes. Your audio and notes stay on this device.
        </p>
        <div className="mt-6 text-sm" aria-live="polite">
          {state.status === 'loading' && <p className="text-ink-faint">Starting…</p>}
          {state.status === 'error' && (
            <p role="alert" className="text-danger">
              Could not reach the app core: {state.message}
            </p>
          )}
          {state.status === 'ready' && (
            <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
              <dt className="text-ink-faint">Version</dt>
              <dd>{state.info.version}</dd>
              <dt className="text-ink-faint">Platform</dt>
              <dd>
                {state.info.os} ({state.info.arch})
              </dd>
              <dt className="text-ink-faint">Data folder</dt>
              <dd className="break-all">{state.info.dataDir}</dd>
            </dl>
          )}
        </div>
      </section>
    </main>
  );
}
