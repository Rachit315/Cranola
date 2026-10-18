# Progress

One entry per milestone: what shipped, how it was verified, and what is known to be missing.

## M0: Environment and scaffolding (2026-10-18)

**Shipped**
- Monorepo at `C:\dev\scribewell`: npm workspace (`apps/desktop`) + Cargo workspace (`apps/desktop/src-tauri`, `crates/*`).
- Tauri 2.11 shell with React 19.3, TypeScript 6.0 (strict + `noUncheckedIndexedAccess` + `exactOptionalPropertyTypes`), Vite 8, Tailwind 4.3 with semantic color tokens for light and dark.
- `sw-core` crate: `AppError` with stable codes, serialized to `{code, message, detail, retryable}` for the UI; `AppInfo`.
- First IPC command `get_app_info` (creates and reports the app data folder) wired to the shell window, including the error path.
- Our own logo (`apps/desktop/src-tauri/icons/logo.svg`) and generated icon set.
- Locked-down CSP (no inline scripts or styles in production; dev-only relaxation for Vite HMR) and a minimal capability set (`core:default`).
- Quality gate `npm run verify`: tsc, ESLint (strict type-checked + jsx-a11y, `--max-warnings 0`), Prettier, Vitest, rustfmt, clippy (`-D warnings`, `unwrap`/`expect`/`panic`/`todo` denied outside tests), cargo test, `tauri build --no-bundle`, with `RUSTFLAGS=-D warnings`.
- GitHub Actions CI running the same gate on Windows, macOS and Ubuntu.
- MIT LICENSE.

**Verified**
- `npm run verify`: all 8 steps pass with zero errors and zero warnings (tsc, eslint, prettier, vitest 2 tests, rustfmt, clippy, cargo test 3 tests, release build).
- Release binary `target/release/scribewell.exe` launched for real and inspected through WebView2's DevTools protocol (`scripts/inspect-webview.mjs`): the page renders under the strict production CSP, follows the system dark theme, and `get_app_info` round-trips to Rust. The data folder `%APPDATA%\dev.scribewell.desktop` did not exist before launch and was created by the command.
- `npm run dev` (Vite HMR + debug build) launched and inspected the same way.
- Error path: covered by a unit test with mocked IPC, not exercised in the real app (making the data folder unwritable on purpose was not worth the risk at M0).
- Not verified: CI. The workflow is written but has not run, because the repo has no GitHub remote yet.

**Toolchain installed on this machine**: VS 2022 Build Tools (C++ x64 + Windows 11 SDK 22621), rustup with stable 1.99.1 (MSVC).

**Deliberate version choices**
- TypeScript pinned to 6.0.x: typescript-eslint 8.71 supports TS < 6.1, and TS 7 is not yet supported by the lint stack.
- ESLint pinned to 9.39 (npm flags 9.x as end-of-life) because eslint-plugin-jsx-a11y does not yet support ESLint 10. Revisit when it does; accessibility lint is worth more than the newest ESLint major right now.
