import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button';
import { ErrorNotice } from '../../components/ErrorNotice';
import { TextAreaField, TextField } from '../../components/Field';
import type { ProfileSettings } from '../../ipc/generated/ProfileSettings';
import { useUpdateSettings } from './useSettings';

interface ProfileFormProps {
  initial: ProfileSettings;
  submitLabel: string;
  onSaved?: (() => void) | undefined;
  secondaryAction?: React.ReactNode;
}

export function ProfileForm({ initial, submitLabel, onSaved, secondaryAction }: ProfileFormProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<ProfileSettings>(initial);
  const update = useUpdateSettings();
  const set = (field: keyof ProfileSettings) => (value: string) => {
    setDraft((d) => ({ ...d, [field]: value }));
  };

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(e) => {
        e.preventDefault();
        update.mutate({ section: 'profile', value: draft }, { onSuccess: () => onSaved?.() });
      }}
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <TextField
          label={t('profile.name')}
          value={draft.name}
          maxLength={120}
          autoComplete="name"
          onChange={(e) => {
            set('name')(e.target.value);
          }}
        />
        <TextField
          label={t('profile.role')}
          placeholder={t('profile.rolePlaceholder')}
          value={draft.role}
          maxLength={120}
          onChange={(e) => {
            set('role')(e.target.value);
          }}
        />
      </div>
      <TextField
        label={t('profile.company')}
        value={draft.company}
        maxLength={120}
        autoComplete="organization"
        onChange={(e) => {
          set('company')(e.target.value);
        }}
      />
      <TextAreaField
        label={t('profile.about')}
        hint={t('profile.aboutHint')}
        placeholder={t('profile.aboutPlaceholder')}
        value={draft.about}
        maxLength={2000}
        onChange={(e) => {
          set('about')(e.target.value);
        }}
      />
      {update.error && <ErrorNotice error={update.error} />}
      <div className="flex items-center gap-3">
        <Button type="submit" variant="primary" busy={update.isPending}>
          {submitLabel}
        </Button>
        {secondaryAction}
        {update.isSuccess && !onSaved && (
          <span className="text-sm text-ink-muted" role="status">
            {t('common.saved')}
          </span>
        )}
      </div>
    </form>
  );
}
