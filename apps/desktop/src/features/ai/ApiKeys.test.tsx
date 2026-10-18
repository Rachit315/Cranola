import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { installFakeBackend } from '../../test/fakeBackend';
import { renderWithProviders } from '../../test/render';
import { ApiKeys } from './ApiKeys';

describe('ApiKeys', () => {
  it('stores a key write-only and shows only its last four characters', async () => {
    const backend = installFakeBackend();
    renderWithProviders(<ApiKeys />);

    const input = await screen.findByLabelText('Anthropic');
    fireEvent.change(input, { target: { value: '  sk-ant-test-1234567890wxyz  ' } });
    const row = input.closest('li');
    if (!row) throw new Error('row not found');
    fireEvent.click(within(row).getByRole('button', { name: 'Save' }));

    expect(await within(row).findByText('Saved ····wxyz')).toBeDefined();
    expect(backend.secrets.get('anthropicApiKey')).toBe('sk-ant-test-1234567890wxyz');
    expect((input as HTMLInputElement).value).toBe('');
    expect(document.body.textContent).not.toContain('sk-ant-test');

    fireEvent.click(within(row).getByRole('button', { name: 'Remove' }));
    await waitFor(() => {
      expect(within(row).queryByText(/Saved/)).toBeNull();
    });
    expect(backend.secrets.has('anthropicApiKey')).toBe(false);
  });

  it('shows a keychain error instead of pretending to save', async () => {
    const backend = installFakeBackend();
    backend.failNext.set('set_secret', {
      code: 'secrets.unavailable',
      message: 'The system keychain is not available',
      detail: 'no secret service',
      retryable: false,
      action: null,
    });
    renderWithProviders(<ApiKeys />);
    const input = await screen.findByLabelText('Groq');
    fireEvent.change(input, { target: { value: 'gsk_abcdefghijklmnop' } });
    const row = input.closest('li');
    if (!row) throw new Error('row not found');
    fireEvent.click(within(row).getByRole('button', { name: 'Save' }));

    expect(
      await within(row).findByText(
        "Your system keychain isn't available, so keys can't be stored safely.",
      ),
    ).toBeDefined();
    expect(within(row).queryByText(/Saved/)).toBeNull();
  });
});
