/** Serialized form of `sw_core::error::AppError`, as Tauri rejects with it. */
export interface AppErrorPayload {
  code: string;
  message: string;
  detail: string | null;
  retryable: boolean;
  action: string | null;
}

/** A real `Error` carrying the Rust error's stable code and recovery hints. */
export class AppError extends Error implements AppErrorPayload {
  readonly code: string;
  readonly detail: string | null;
  readonly retryable: boolean;
  readonly action: string | null;

  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.name = 'AppError';
    this.code = payload.code;
    this.detail = payload.detail;
    this.retryable = payload.retryable;
    this.action = payload.action;
  }
}

export function isAppErrorPayload(value: unknown): value is AppErrorPayload {
  if (typeof value !== 'object' || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.code === 'string' && typeof v.message === 'string';
}

/** Normalizes anything thrown by `invoke` (or by our own code) into an AppError. */
export function toAppError(value: unknown): AppError {
  if (value instanceof AppError) return value;
  if (isAppErrorPayload(value)) {
    return new AppError({
      code: value.code,
      message: value.message,
      detail: typeof value.detail === 'string' ? value.detail : null,
      retryable: typeof value.retryable === 'boolean' && value.retryable,
      action: typeof value.action === 'string' ? value.action : null,
    });
  }
  const message = value instanceof Error ? value.message : String(value);
  return new AppError({ code: 'internal', message, detail: null, retryable: false, action: null });
}
