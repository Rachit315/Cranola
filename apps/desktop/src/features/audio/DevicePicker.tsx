import { useTranslation } from 'react-i18next';

import { ErrorNotice } from '../../components/ErrorNotice';
import { SelectField } from '../../components/Field';
import type { AudioDevice } from '../../ipc/generated/AudioDevice';
import type { AudioSettings } from '../../ipc/generated/AudioSettings';
import { useAudioDevices } from './queries';

const SYSTEM_DEFAULT = '';

interface DevicePickerProps {
  value: AudioSettings;
  onChange: (next: AudioSettings) => void;
}

export function DevicePicker({ value, onChange }: DevicePickerProps) {
  const { t } = useTranslation();
  const devices = useAudioDevices();

  if (devices.error) {
    return (
      <ErrorNotice
        error={devices.error}
        onRetry={() => {
          void devices.refetch();
        }}
      />
    );
  }

  const options = (direction: AudioDevice['direction']) => {
    const list = (devices.data ?? []).filter((d) => d.direction === direction);
    const defaultName = list.find((d) => d.isDefault)?.name;
    return [
      {
        value: SYSTEM_DEFAULT,
        label: defaultName
          ? t('audio.devices.systemDefaultNamed', { name: defaultName })
          : t('audio.devices.systemDefault'),
      },
      ...list.map((d) => ({ value: d.id, label: d.name })),
    ];
  };

  return (
    <div className="grid gap-4 sm:grid-cols-2">
      <SelectField
        label={t('audio.devices.microphone')}
        hint={t('audio.devices.microphoneHint')}
        options={options('input')}
        value={value.inputDeviceId ?? SYSTEM_DEFAULT}
        disabled={devices.isPending}
        onChange={(e) => {
          onChange({ ...value, inputDeviceId: e.target.value || null });
        }}
      />
      <SelectField
        label={t('audio.devices.output')}
        hint={t('audio.devices.outputHint')}
        options={options('output')}
        value={value.outputDeviceId ?? SYSTEM_DEFAULT}
        disabled={devices.isPending}
        onChange={(e) => {
          onChange({ ...value, outputDeviceId: e.target.value || null });
        }}
      />
    </div>
  );
}
