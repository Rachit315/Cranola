import { describe, expect, it } from 'vitest';

import { applyTheme } from '../app/theme';
import { formatBytes } from './ai/queries';
import { levelPercent } from './audio/queries';
import { greetingKey } from './home/greeting';

describe('helpers', () => {
  it('maps decibels to a level bar', () => {
    expect(levelPercent(0)).toBe(100);
    expect(levelPercent(-30)).toBe(50);
    expect(levelPercent(-60)).toBe(0);
    expect(levelPercent(-120)).toBe(0);
    expect(levelPercent(6)).toBe(100);
  });

  it('formats model sizes', () => {
    expect(formatBytes(5_200_000_000)).toBe('5.2 GB');
    expect(formatBytes(400_000_000)).toBe('400 MB');
    expect(formatBytes(0)).toBe('');
  });

  it('picks a greeting by hour', () => {
    expect(greetingKey(8)).toBe('home.morning');
    expect(greetingKey(13)).toBe('home.afternoon');
    expect(greetingKey(21)).toBe('home.evening');
  });

  it('applies and clears forced themes', () => {
    const root = document.createElement('div');
    applyTheme('dark', root);
    expect(root.dataset.theme).toBe('dark');
    applyTheme('system', root);
    expect(root.dataset.theme).toBeUndefined();
  });
});
