import clsx from 'clsx';
import { useId } from 'react';
import type {
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from 'react';

const control =
  'w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink placeholder:text-ink-faint ' +
  'focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-accent/30';

interface FieldShellProps {
  label: string;
  hint?: string | undefined;
  id: string;
  children: ReactNode;
}

function FieldShell({ label, hint, id, children }: FieldShellProps) {
  return (
    <div className="flex flex-col gap-1.5">
      <label htmlFor={id} className="text-sm font-medium text-ink">
        {label}
      </label>
      {children}
      {hint && (
        <p id={`${id}-hint`} className="text-xs text-ink-faint">
          {hint}
        </p>
      )}
    </div>
  );
}

type TextFieldProps = InputHTMLAttributes<HTMLInputElement> & { label: string; hint?: string };

export function TextField({ label, hint, className, ...rest }: TextFieldProps) {
  const id = useId();
  return (
    <FieldShell label={label} hint={hint} id={id}>
      <input
        id={id}
        aria-describedby={hint ? `${id}-hint` : undefined}
        className={clsx(control, 'h-9', className)}
        {...rest}
      />
    </FieldShell>
  );
}

type TextAreaFieldProps = TextareaHTMLAttributes<HTMLTextAreaElement> & {
  label: string;
  hint?: string;
};

export function TextAreaField({ label, hint, className, ...rest }: TextAreaFieldProps) {
  const id = useId();
  return (
    <FieldShell label={label} hint={hint} id={id}>
      <textarea
        id={id}
        aria-describedby={hint ? `${id}-hint` : undefined}
        className={clsx(control, 'min-h-24 py-2', className)}
        {...rest}
      />
    </FieldShell>
  );
}

type SelectFieldProps = SelectHTMLAttributes<HTMLSelectElement> & {
  label: string;
  hint?: string;
  options: { value: string; label: string }[];
};

export function SelectField({ label, hint, options, className, ...rest }: SelectFieldProps) {
  const id = useId();
  return (
    <FieldShell label={label} hint={hint} id={id}>
      <select
        id={id}
        aria-describedby={hint ? `${id}-hint` : undefined}
        className={clsx(control, 'h-9', className)}
        {...rest}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </FieldShell>
  );
}
