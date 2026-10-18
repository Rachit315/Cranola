export const STEPS = [
  'welcome',
  'privacy',
  'microphone',
  'systemAudio',
  'ai',
  'profile',
  'done',
] as const;

export type Step = (typeof STEPS)[number];
