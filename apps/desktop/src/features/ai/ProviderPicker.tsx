import { useTranslation } from 'react-i18next';

import type { LlmProvider } from '../../ipc/generated/LlmProvider';

const PROVIDERS: LlmProvider[] = ['ollama', 'anthropic', 'openai', 'gemini', 'openrouter'];

interface ProviderPickerProps {
  value: LlmProvider | null;
  onChange: (provider: LlmProvider) => void;
}

export function ProviderPicker({ value, onChange }: ProviderPickerProps) {
  const { t } = useTranslation();
  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="mb-1 text-sm font-medium text-ink">{t('ai.provider.legend')}</legend>
      {PROVIDERS.map((p) => {
        const id = `llm-provider-${p}`;
        return (
          <div
            key={p}
            className="relative flex items-start gap-3 rounded-lg border border-line bg-surface p-3 has-[:checked]:border-accent has-[:checked]:bg-accent-soft"
          >
            <input
              id={id}
              type="radio"
              name="llm-provider"
              value={p}
              checked={value === p}
              aria-describedby={`${id}-desc`}
              onChange={() => {
                onChange(p);
              }}
              className="mt-1 accent-accent"
            />
            <div>
              {/* The label is stretched over the whole card so it is all clickable. */}
              <label
                htmlFor={id}
                className="block cursor-pointer text-sm font-medium text-ink after:absolute after:inset-0"
              >
                {t(`ai.provider.${p}.label`)}
              </label>
              <p id={`${id}-desc`} className="text-xs text-ink-muted">
                {t(`ai.provider.${p}.description`)}
              </p>
            </div>
          </div>
        );
      })}
    </fieldset>
  );
}
