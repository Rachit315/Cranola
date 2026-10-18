import { mockIPC } from '@tauri-apps/api/mocks';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { App } from './App';

describe('App', () => {
  it('shows app info returned by the core', async () => {
    mockIPC((cmd) => {
      if (cmd === 'get_app_info') {
        return {
          name: 'Scribewell',
          version: '0.1.0',
          os: 'windows',
          arch: 'x86_64',
          dataDir: 'C:\\Users\\test\\AppData\\Roaming\\dev.scribewell.desktop',
        };
      }
      throw new Error(`unexpected command ${cmd}`);
    });

    render(<App />);

    expect(await screen.findByText('0.1.0')).toBeDefined();
    expect(screen.getByText('windows (x86_64)')).toBeDefined();
  });

  it('shows the core error message when the command fails', async () => {
    // Tauri rejects with the serialized AppError when a command returns Err.
    const ipcError: unknown = {
      code: 'io.data_dir',
      message: 'The data folder is not writable',
      detail: null,
      retryable: false,
    };
    mockIPC(() => Promise.reject(ipcError));

    render(<App />);

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('The data folder is not writable');
  });
});
