import { useMutation, useQueryClient } from '@tanstack/react-query';
import { KeyRound } from 'lucide-react';
import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button';
import { ErrorNotice } from '../../components/ErrorNotice';
import { commands } from '../../ipc/commands';
import type { AppError } from '../../ipc/errors';
import type { SecretKey } from '../../ipc/generated/SecretKey';
import type { SecretStatus } from '../../ipc/generated/SecretStatus';
import { secretsKey, useSecretStatus } from './queries';

interface KeyRowProps {
  secretKey: SecretKey;
  label: string;
  description: string;
  status: SecretStatus | undefined;
}

function KeyRow({ secretKey, label, description, status }: KeyRowProps) {
  const { t } = useTranslation();
  const client = useQueryClient();
  const inputId = useId();
  const [draft, setDraft] = useState('');

  const updateCache = (next: SecretStatus) => {
    client.setQueryData<SecretStatus[]>(secretsKey, (prev) =>
      (prev ?? []).map((s) => (s.key === next.key ? next : s)),
    );
  };
  const save = useMutation<SecretStatus, AppError, string>({
    mutationFn: (value) => commands.setSecret(secretKey, value),
    onSuccess: (next) => {
      setDraft('');
      updateCache(next);
    },
  });
  const remove = useMutation<SecretStatus, AppError>({
    mutationFn: () => commands.deleteSecret(secretKey),
    onSuccess: updateCache,
  });

  const configured = status?.configured === true;

  return (
    <li className="flex flex-col gap-2 py-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <label htmlFor={inputId} className="text-sm font-medium text-ink">
            {label}
          </label>
          <p className="text-xs text-ink-faint">{description}</p>
        </div>
        {configured && (
          <span className="flex items-center gap-1.5 rounded-full bg-accent-soft px-2.5 py-1 text-xs text-ink">
            <KeyRound className="size-3" aria-hidden />
            {t('ai.keys.storedHint', { hint: status.hint ?? '' })}
          </span>
        )}
      </div>
      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (draft.trim()) save.mutate(draft);
        }}
      >
        <input
          id={inputId}
          type="password"
          autoComplete="off"
          spellCheck={false}
          value={draft}
          placeholder={configured ? t('ai.keys.replacePlaceholder') : t('ai.keys.pastePlaceholder')}
          onChange={(e) => {
            setDraft(e.target.value);
          }}
          className="h-9 flex-1 rounded-lg border border-line bg-surface px-3 font-mono text-sm text-ink placeholder:font-sans placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-accent/30"
        />
        <Button type="submit" variant="primary" busy={save.isPending} disabled={!draft.trim()}>
          {t('common.save')}
        </Button>
        {configured && (
          <Button
            variant="danger"
            busy={remove.isPending}
            onClick={() => {
              remove.mutate();
            }}
          >
            {t('common.remove')}
          </Button>
        )}
      </form>
      {save.error && <ErrorNotice error={save.error} />}
      {remove.error && <ErrorNotice error={remove.error} />}
    </li>
  );
}

const ROWS: { key: SecretKey; group: 'llm' | 'stt' }[] = [
  { key: 'anthropicApiKey', group: 'llm' },
  { key: 'openaiApiKey', group: 'llm' },
  { key: 'geminiApiKey', group: 'llm' },
  { key: 'openrouterApiKey', group: 'llm' },
  { key: 'deepgramApiKey', group: 'stt' },
  { key: 'groqApiKey', group: 'stt' },
];

export function ApiKeys() {
  const { t } = useTranslation();
  const statuses = useSecretStatus();

  if (statuses.error) return <ErrorNotice error={statuses.error} />;

  return (
    <div className="flex flex-col gap-1">
      <p className="text-sm text-ink-muted">{t('ai.keys.explainer')}</p>
      <ul className="divide-y divide-line">
        {ROWS.map(({ key }) => (
          <KeyRow
            key={key}
            secretKey={key}
            label={t(`ai.keys.providers.${key}.label`)}
            description={t(`ai.keys.providers.${key}.description`)}
            status={statuses.data?.find((s) => s.key === key)}
          />
        ))}
      </ul>
    </div>
  );
}
