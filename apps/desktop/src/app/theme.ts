import { useEffect } from 'react';

import type { Theme } from '../ipc/generated/Theme';

/** Applies the chosen theme; "system" defers to prefers-color-scheme in CSS. */
export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement) {
  if (theme === 'system') {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = theme;
  }
}

export function useApplyTheme(theme: Theme | undefined) {
  useEffect(() => {
    if (theme) applyTheme(theme);
  }, [theme]);
}
