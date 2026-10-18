// Typed wrappers for every Rust command. Argument names are camelCase; Tauri
// maps them to the snake_case parameters of the Rust functions. Every wrapper
// rejects with an `AppError`, never with a raw IPC payload.
import { invoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';

import { toAppError } from './errors';
import type { AppInfo } from './generated/AppInfo';
import type { AudioDevice } from './generated/AudioDevice';
import type { FrontendLogLevel } from './generated/FrontendLogLevel';
import type { LoopbackCheck } from './generated/LoopbackCheck';
import type { MicCheck } from './generated/MicCheck';
import type { MicPermission } from './generated/MicPermission';
import type { OllamaStatus } from './generated/OllamaStatus';
import type { OpenTarget } from './generated/OpenTarget';
import type { SecretKey } from './generated/SecretKey';
import type { SecretStatus } from './generated/SecretStatus';
import type { Settings } from './generated/Settings';
import type { SettingsUpdate } from './generated/SettingsUpdate';
import type { StartupStatus } from './generated/StartupStatus';

async function call<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    throw toAppError(err);
  }
}

export const commands = {
  getStartupStatus: () => call<StartupStatus>('get_startup_status'),
  getAppInfo: () => call<AppInfo>('get_app_info'),
  openLocation: (target: OpenTarget) => call<null>('open_location', { target }),
  logFrontend: (level: FrontendLogLevel, message: string) =>
    call<null>('log_frontend', { level, message }),

  getSettings: () => call<Settings>('get_settings'),
  updateSettings: (update: SettingsUpdate) => call<Settings>('update_settings', { update }),
  completeOnboarding: () => call<Settings>('complete_onboarding'),

  listSecretStatus: () => call<SecretStatus[]>('list_secret_status'),
  setSecret: (key: SecretKey, value: string) => call<SecretStatus>('set_secret', { key, value }),
  deleteSecret: (key: SecretKey) => call<SecretStatus>('delete_secret', { key }),

  listAudioDevices: () => call<AudioDevice[]>('list_audio_devices'),
  getMicrophonePermission: () => call<MicPermission>('get_microphone_permission'),
  checkMicrophone: (deviceId: string | null) => call<MicCheck>('check_microphone', { deviceId }),
  checkSystemAudio: (deviceId: string | null) =>
    call<LoopbackCheck>('check_system_audio', { deviceId }),

  detectOllama: (baseUrl: string | null) => call<OllamaStatus>('detect_ollama', { baseUrl }),
};
