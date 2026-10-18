import { NotebookPen } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { Settings } from '../../ipc/generated/Settings';
import { greetingKey } from './greeting';

export function HomePage({ settings }: { settings: Settings }) {
  const { t } = useTranslation();
  const firstName = settings.profile.name.trim().split(/\s+/)[0] ?? '';
  const greeting = t(greetingKey(new Date().getHours()));

  return (
    <div className="mx-auto w-full max-w-3xl px-8 py-10">
      <h1 className="font-serif text-3xl text-ink">
        {firstName ? t('home.greetingNamed', { greeting, name: firstName }) : greeting}
      </h1>
      <div className="mt-10 flex flex-col items-center rounded-xl border border-dashed border-line px-6 py-16 text-center">
        <NotebookPen className="size-8 text-ink-faint" aria-hidden />
        <h2 className="mt-4 text-base font-medium text-ink">{t('home.empty.title')}</h2>
        <p className="mt-1 max-w-sm text-sm text-ink-muted">{t('home.empty.body')}</p>
      </div>
    </div>
  );
}
