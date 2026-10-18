import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { installFakeBackend } from '../test/fakeBackend';
import { renderWithProviders } from '../test/render';
import { App } from './App';

const click = (name: string | RegExp) => {
  fireEvent.click(screen.getByRole('button', { name }));
};

/** Waits for a button to appear, then clicks it. */
const clickWhenReady = async (name: string | RegExp) => {
  fireEvent.click(await screen.findByRole('button', { name }));
};

describe('App startup', () => {
  it('shows the startup error screen when the core failed to start', async () => {
    const backend = installFakeBackend();
    backend.startup = {
      ready: false,
      error: {
        code: 'db.migration_failed',
        message: 'Upgrading the local database failed',
        detail: 'migration 2: no such table',
      },
      logDir: 'C:\\logs',
    };
    renderWithProviders(<App />);

    expect(await screen.findByText("Scribewell couldn't start")).toBeDefined();
    expect(screen.getByText('migration 2: no such table')).toBeDefined();
    click('Open logs folder');
    await waitFor(() => {
      expect(backend.calls.some((c) => c.cmd === 'open_location')).toBe(true);
    });
  });

  it('goes straight to home when onboarding is done', async () => {
    installFakeBackend({
      onboarding: { completedAt: 1 },
      profile: { name: 'Ada Lovelace', role: '', company: '', about: '' },
    });
    renderWithProviders(<App />);
    expect(await screen.findByRole('heading', { level: 1, name: /Ada$/ })).toBeDefined();
    expect(screen.getByText('No meetings yet')).toBeDefined();
  });

  it('applies the saved theme to the document', async () => {
    installFakeBackend({ onboarding: { completedAt: 1 }, general: { theme: 'dark' } });
    renderWithProviders(<App />);
    await screen.findByText('No meetings yet');
    expect(document.documentElement.dataset.theme).toBe('dark');
  });
});

describe('Onboarding', () => {
  it('walks through every step and lands on home', async () => {
    const backend = installFakeBackend();
    renderWithProviders(<App />);

    await clickWhenReady('Get started');
    expect(await screen.findByText('How Scribewell handles your data')).toBeDefined();
    click('Continue');

    // Microphone: skippable until the check passes.
    expect(await screen.findByText('Check your microphone')).toBeDefined();
    expect(screen.getByRole('button', { name: 'Skip for now' })).toBeDefined();
    click('Test microphone');
    expect(await screen.findByText('Heard you through USB Microphone.')).toBeDefined();
    click('Continue');

    // System audio loopback test.
    expect(await screen.findByText("Check your computer's sound")).toBeDefined();
    click('Play test sound');
    expect(await screen.findByText(/Captured the test sound from Headphones/)).toBeDefined();
    click('Continue');

    // AI provider: picking Ollama saves the choice and shows its status.
    expect(await screen.findByText('Choose an AI model')).toBeDefined();
    fireEvent.click(screen.getByLabelText('Ollama (on this computer)'));
    expect(await screen.findByText('Ollama 0.40.3 is running')).toBeDefined();
    expect(screen.getByText('qwen3:8b')).toBeDefined();
    expect(backend.settings.ai.llmProvider).toBe('ollama');
    click('Continue');

    // Profile.
    expect(await screen.findByText('Tell Scribewell about you')).toBeDefined();
    fireEvent.change(screen.getByLabelText('Your name'), { target: { value: 'Grace Hopper' } });
    click('Continue');
    expect(await screen.findByText("You're all set")).toBeDefined();
    expect(backend.settings.profile.name).toBe('Grace Hopper');

    click('Start using Scribewell');
    expect(await screen.findByRole('heading', { level: 1, name: /Grace$/ })).toBeDefined();
    expect(backend.settings.onboarding.completedAt).not.toBeNull();
  });

  it('explains a blocked microphone and offers the privacy settings', async () => {
    const backend = installFakeBackend();
    backend.failNext.set('check_microphone', {
      code: 'audio.permission_denied',
      message: 'Microphone access is blocked by system privacy settings',
      detail: null,
      retryable: false,
      action: 'open_settings:microphone',
    });
    renderWithProviders(<App />);
    await clickWhenReady('Get started');
    await clickWhenReady('Continue');
    await clickWhenReady('Test microphone');

    expect(
      await screen.findByText('Microphone access is turned off in your system privacy settings.'),
    ).toBeDefined();
    click('Open privacy settings');
    await waitFor(() => {
      const open = backend.calls.find((c) => c.cmd === 'open_location');
      expect(open?.args).toEqual({ target: 'microphoneSettings' });
    });
    // The step stays skippable so a denied microphone never blocks setup.
    expect(screen.getByRole('button', { name: 'Skip for now' })).toBeDefined();
  });
});
