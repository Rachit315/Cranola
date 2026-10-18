// An in-memory stand-in for the Rust side, plugged in through Tauri's IPC
// mock. It mirrors the real commands' contracts (shapes, write-only secrets,
// error payloads) so UI flows can be tested end to end without a backend.
import { mockIPC } from '@tauri-apps/api/mocks';

import type { AppErrorPayload } from '../ipc/errors';
import type { SecretKey } from '../ipc/generated/SecretKey';
import type { Settings } from '../ipc/generated/Settings';
import type { SettingsUpdate } from '../ipc/generated/SettingsUpdate';
import type { StartupStatus } from '../ipc/generated/StartupStatus';

export function defaultSettings(): Settings {
  return {
    general: { theme: 'system' },
    profile: { name: '', role: '', company: '', about: '' },
    audio: { inputDeviceId: null, outputDeviceId: null },
    ai: { llmProvider: null, ollamaBaseUrl: 'http://127.0.0.1:11434' },
    onboarding: { completedAt: null },
  };
}

export interface FakeBackend {
  settings: Settings;
  secrets: Map<SecretKey, string>;
  calls: { cmd: string; args: unknown }[];
  startup: StartupStatus;
  failNext: Map<string, AppErrorPayload>;
}

const ALL_KEYS: SecretKey[] = [
  'anthropicApiKey',
  'openaiApiKey',
  'geminiApiKey',
  'openrouterApiKey',
  'deepgramApiKey',
  'groqApiKey',
];

export function installFakeBackend(overrides: Partial<Settings> = {}): FakeBackend {
  const backend: FakeBackend = {
    settings: { ...defaultSettings(), ...overrides },
    secrets: new Map(),
    calls: [],
    startup: { ready: true, error: null, logDir: 'C:\\logs' },
    failNext: new Map(),
  };

  const status = (key: SecretKey) => {
    const value = backend.secrets.get(key);
    return { key, configured: value !== undefined, hint: value ? value.slice(-4) : null };
  };

  mockIPC((cmd, payload) => {
    const args = (payload ?? {}) as Record<string, unknown>;
    backend.calls.push({ cmd, args });
    const failure = backend.failNext.get(cmd);
    if (failure) {
      backend.failNext.delete(cmd);
      const rejection: unknown = failure;
      return Promise.reject(rejection);
    }
    switch (cmd) {
      case 'get_startup_status':
        return backend.startup;
      case 'get_app_info':
        return {
          name: 'Scribewell',
          version: '0.1.0',
          os: 'windows',
          arch: 'x86_64',
          dataDir: 'C:\\data',
          logDir: 'C:\\logs',
        };
      case 'get_settings':
        return backend.settings;
      case 'update_settings': {
        const update = args.update as SettingsUpdate;
        backend.settings = { ...backend.settings, [update.section]: update.value };
        return backend.settings;
      }
      case 'complete_onboarding':
        backend.settings = { ...backend.settings, onboarding: { completedAt: 1_700_000_000_000 } };
        return backend.settings;
      case 'list_secret_status':
        return ALL_KEYS.map(status);
      case 'set_secret': {
        const key = args.key as SecretKey;
        backend.secrets.set(key, String(args.value).trim());
        return status(key);
      }
      case 'delete_secret': {
        const key = args.key as SecretKey;
        backend.secrets.delete(key);
        return status(key);
      }
      case 'list_audio_devices':
        return [
          { id: 'in-1', name: 'USB Microphone', direction: 'input', isDefault: true },
          { id: 'out-1', name: 'Headphones', direction: 'output', isDefault: true },
        ];
      case 'get_microphone_permission':
        return 'allowed';
      case 'check_microphone':
        return {
          deviceName: 'USB Microphone',
          peakDbfs: -18,
          rmsDbfs: -40,
          heardSignal: true,
          allZero: false,
        };
      case 'check_system_audio':
        return { deviceName: 'Headphones', toneDetected: true, toneDbfs: -14, capturedMs: 1500 };
      case 'detect_ollama':
        return {
          baseUrl: 'http://127.0.0.1:11434',
          version: '0.40.3',
          models: [
            { name: 'qwen3:8b', sizeBytes: 5_200_000_000, parameterSize: '8.2B', family: 'qwen3' },
          ],
        };
      case 'open_location':
      case 'log_frontend':
        return null;
      default:
        throw new Error(`fake backend: unhandled command ${cmd}`);
    }
  });

  return backend;
}
