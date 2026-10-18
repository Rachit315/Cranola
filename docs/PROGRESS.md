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

## M1: Foundation (2026-10-18)

**Shipped**
- `sw-db`: SQLite (bundled, FTS5) in WAL mode with foreign keys on. Schema v1 covers every entity in the architecture (meetings, recordings, audio chunks, transcript segments, raw/enhanced documents with versions, note blocks with authorship, citations, templates, recipes, folders, tags, people, companies, calendar, chats, integrations, deliveries, LLM run log, embeddings, FTS index), using STRICT tables.
- Migration runner: all pending migrations in one transaction (a failure leaves the database untouched), a `VACUUM INTO` backup before upgrading an existing database, the last 5 backups kept, refusal to open a database from a newer app version, consecutive-version check.
- Settings: typed sections (general, profile, audio, ai, onboarding) stored as one JSON row each; validation before write; unreadable rows fall back to defaults instead of breaking the app.
- `sw-secrets`: OS keychain wrapper (Windows Credential Manager, macOS Keychain, Linux Secret Service), no plain-text fallback. The UI can only write, delete, and see "configured + last 4 characters".
- `sw-audio` (first part): device listing with stable ids, Windows microphone privacy detection (registry consent store, including the "desktop apps" switch, which otherwise yields silent zeros), a microphone level check, and a system-audio loopback self-test (plays a 1 kHz tone and detects it in the captured output with the Goertzel algorithm).
- `sw-llm` (first part): Ollama detection (version + installed models).
- App: startup that never panics (failures produce a dedicated error screen with the log folder), JSON log files rotated daily and kept 7 days, a redaction layer that masks API-key-shaped strings before anything reaches disk, panic logging, frontend error forwarding.
- UI: shell with sidebar, Home (greeting + empty state), Settings (appearance, profile, audio devices + checks, AI provider + Ollama + API keys, data folders), 7-step onboarding (welcome, privacy, microphone, system audio, AI, profile, done; every step skippable), light/dark/system themes with tokens, bundled fonts (Inter, Source Serif 4), i18n with all strings in `en.json` and typed keys, IPC types generated from Rust with ts-rs.

**Verified**
- Gate: all steps pass with zero warnings. Rust: 68 tests (including a real Windows Credential Manager round trip and real-database migration rollback/backup tests). UI: 15 tests, including a full onboarding walk-through and the blocked-microphone path against a stateful fake backend.
- `cargo run -p sw-audio --example audio_check` on this machine: lists 2 inputs and 2 outputs; mic check heard the USB headset; loopback captured the test tone at -14 dBFS (expected about -14 for amplitude 0.2).
- Real release app, driven through DevTools:
  - First launch created and migrated the database.
  - Onboarding was completed in the real window. You clicked through it yourself; the full flow is also covered by the UI test.
  - Settings: the mic check passed, the system-audio check captured the tone, "Ollama isn't running" was shown correctly, and after starting Ollama it was detected with the installed model.
  - A dummy OpenRouter key saved from the UI appeared only in Credential Manager (not in the database, the logs, or the page, which showed `····wxyz`), and Remove deleted it.
  - Theme switch applied immediately and persisted across a restart.
  - A database marked as coming from a newer app version showed the startup error screen instead of crashing.
- Not exercised in the real app: microphone-permission-denied (needs changing your Windows privacy setting; covered by unit and UI tests), keychain-unavailable (covered by a UI test), and the migration-failure screen (covered by a unit test).

**Incident during verification**
- To simulate a "newer database", I copied `scribewell.db` as a backup while the app (in WAL mode) had been force-killed. The data was still in the `-wal` file, so restoring that copy produced an empty database. The only data lost were 2 settings rows from your onboarding run; I re-created them with their original values. Lesson: never copy a live SQLite file. Use `VACUUM INTO` or the backup API, which is what the app's own migration backups already do.

**Known gaps (by design, later milestones)**
- No recording yet (M2), so Home has only the empty state.
- Model download and transcription choice are not in onboarding yet; they arrive with the model manager (M3).
- Cloud API keys are stored but not yet validated against the provider (M5 adds the provider clients).
- `reqwest` is built without TLS until M5 (rustls with the `ring` provider, to avoid needing CMake/NASM for aws-lc).
