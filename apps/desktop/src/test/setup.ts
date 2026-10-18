import '../i18n';

import { clearMocks } from '@tauri-apps/api/mocks';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => {
  cleanup();
  clearMocks();
  window.location.hash = '';
  delete document.documentElement.dataset.theme;
});
