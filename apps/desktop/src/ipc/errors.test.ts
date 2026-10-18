import { describe, expect, it } from 'vitest';

import { AppError, toAppError } from './errors';

describe('toAppError', () => {
  it('wraps IPC payloads in a real Error', () => {
    const err = toAppError({
      code: 'net.unreachable',
      message: 'Could not reach Ollama',
      detail: 'connection refused',
      retryable: true,
      action: null,
    });
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(AppError);
    expect(err.code).toBe('net.unreachable');
    expect(err.retryable).toBe(true);
    expect(err.detail).toBe('connection refused');
  });

  it('fills missing optional fields defensively', () => {
    const err = toAppError({ code: 'internal', message: 'x' });
    expect(err.detail).toBeNull();
    expect(err.retryable).toBe(false);
    expect(err.action).toBeNull();
  });

  it('converts plain errors and strings', () => {
    expect(toAppError(new Error('boom')).message).toBe('boom');
    expect(toAppError('bad').code).toBe('internal');
  });

  it('passes AppError through unchanged', () => {
    const original = toAppError({ code: 'db.error', message: 'm' });
    expect(toAppError(original)).toBe(original);
  });
});
