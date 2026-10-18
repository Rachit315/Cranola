import clsx from 'clsx';
import { Home, Settings as SettingsIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { NavLink, Outlet } from 'react-router';

import { Logo } from '../components/Logo';

function NavItem({ to, icon, label }: { to: string; icon: ReactNode; label: string }) {
  return (
    <NavLink
      to={to}
      end
      className={({ isActive }) =>
        clsx(
          'flex items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm transition',
          'focus-visible:outline-2 focus-visible:outline-accent',
          isActive
            ? 'bg-accent-soft font-medium text-ink'
            : 'text-ink-muted hover:bg-line/40 hover:text-ink',
        )
      }
    >
      {icon}
      {label}
    </NavLink>
  );
}

export function Shell() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full">
      <nav
        aria-label={t('nav.label')}
        className="flex w-56 shrink-0 flex-col border-r border-line bg-surface px-3 py-4"
      >
        <div className="mb-6 flex items-center gap-2 px-2.5">
          <Logo className="size-6" />
          <span className="font-serif text-lg text-ink">{t('app.name')}</span>
        </div>
        <div className="flex flex-col gap-0.5">
          <NavItem to="/" icon={<Home className="size-4" aria-hidden />} label={t('nav.home')} />
        </div>
        <div className="mt-auto flex flex-col gap-0.5">
          <NavItem
            to="/settings"
            icon={<SettingsIcon className="size-4" aria-hidden />}
            label={t('nav.settings')}
          />
        </div>
      </nav>
      <main className="flex-1 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  );
}
