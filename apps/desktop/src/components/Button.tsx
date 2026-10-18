import clsx from 'clsx';
import { Loader2 } from 'lucide-react';
import type { ButtonHTMLAttributes, ReactNode } from 'react';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  busy?: boolean;
  icon?: ReactNode;
}

const variants: Record<Variant, string> = {
  primary: 'bg-accent text-on-accent hover:brightness-110 disabled:opacity-50',
  secondary: 'border border-line bg-surface text-ink hover:bg-paper disabled:opacity-50',
  ghost: 'text-ink-muted hover:bg-line/40 hover:text-ink disabled:opacity-50',
  danger: 'border border-line bg-surface text-danger hover:bg-danger/10 disabled:opacity-50',
};

export function Button({
  variant = 'secondary',
  busy = false,
  icon,
  className,
  children,
  disabled,
  type = 'button',
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={clsx(
        'inline-flex h-9 items-center justify-center gap-2 rounded-lg px-3.5 text-sm font-medium transition',
        'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent',
        'disabled:cursor-not-allowed',
        variants[variant],
        className,
      )}
      disabled={disabled === true || busy}
      aria-busy={busy || undefined}
      {...rest}
    >
      {busy ? <Loader2 className="size-4 animate-spin" aria-hidden /> : icon}
      {children}
    </button>
  );
}
