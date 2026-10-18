# Scribewell: architecture and delivery plan

Status: draft for approval. Companion to `PRD.md` and `teardown.md`.

## 1. Stack

| Layer | Choice | Why |
|---|---|---|
| Shell | **Tauri 2** | Small binaries, native webview, Rust backend for audio/ML/secrets, first-class Windows/macOS/Linux. As specified. |
| UI | **React 19 + TypeScript 5 (strict) + Vite** | As specified. |
| Styling | **Tailwind CSS v4** with CSS-variable design tokens | As specified; tokens give us light/dark and our own identity. |
| Editor | **TipTap 3 (ProseMirror)** | Mature rich-text engine; node attributes and marks let us carry authorship (`origin`) and citations per block/span; markdown input rules; collaborative-ready schema. |
| UI state | Zustand (UI state) + TanStack Query (IPC data cache) | Small, testable; no global Redux. |
| Command palette | cmdk | Accessible, unstyled. |
| i18n | i18next + react-i18next | Standard, supports pseudo-locale. |
| Icons | lucide-react (ISC) | Our own icon language; nothing derived from Granola. |
| Fonts | Bundled locally (OFL): "Inter" for UI, "Newsreader" for note body | Offline, distinct from Granola's look. |
| Database | **SQLite via rusqlite (bundled, FTS5 enabled), WAL mode** | As specified. Access lives in Rust, not via `tauri-plugin-sql`, so FTS indexing, migrations, pipelines and the UI share one consistent data layer and the webview never runs raw SQL. |
| Audio I/O | cpal (mic everywhere; WASAPI loopback on Windows), rubato (resampling), hound (WAV), flacenc (pure-Rust FLAC for compact storage), symphonia (decode imports) | Pure Rust where possible, so no extra C toolchains beyond whisper.cpp. |
| macOS system audio | Core Audio process tap (macOS 14.2+), ScreenCaptureKit fallback (13+) | Official Apple APIs. |
| Linux system audio | PulseAudio monitor source via libpulse (works on PipeWire through pipewire-pulse) | Covers modern distros. |
| Local STT | **whisper.cpp via whisper-rs**, Vulkan feature on Windows/Linux, Metal on macOS, CPU fallback | As specified; Vulkan works on NVIDIA/AMD/Intel without CUDA. |
| VAD | Silero VAD through whisper.cpp's built-in VAD; fallback to webrtc-vad if the binding proves insufficient (decided in an M3 spike) | Small, accurate, no ONNX runtime dependency. |
| Cloud STT | Deepgram, OpenAI, Groq (reqwest) | BYOK. |
| LLM | Rust `llm` crate: Ollama, Anthropic, OpenAI, Gemini, OpenRouter adapters; streaming via SSE/NDJSON; tokens to UI over `tauri::ipc::Channel` | Keys never leave Rust; no CORS issues; testable with a mock HTTP server. |
| Secrets | `keyring` crate (Windows Credential Manager, macOS Keychain, Linux Secret Service) | As specified; if no Secret Service on Linux, we refuse to store and explain. Never a plain-text fallback. |
| OAuth | `oauth2` crate, PKCE, loopback redirect on 127.0.0.1 random port | Desktop-app best practice for Google and Microsoft. |
| Type sharing | `ts-rs` generates TS types from Rust structs; thin hand-written typed `invoke` wrappers | Stable, no RC dependencies. |
| Logging | `tracing` + rolling JSON files, secret-redaction layer | See §8. |
| Tests | cargo test, wiremock, insta (prompt snapshots); vitest + Testing Library; Playwright against the Vite build with mocked Tauri IPC; tauri-driver smoke test on Windows | See §9. |
| Lint/format | clippy `-D warnings`, rustfmt; ESLint (typescript-eslint strict-type-checked) `--max-warnings 0`, Prettier; `tsc --noEmit` | Zero-warning gate. |
| Share server | Rust axum single binary + Dockerfile, in `share-server/` | Optional, self-hosted. |
| CI/CD | GitHub Actions + `tauri-action` matrix | Installers for all three OSes. |

No changes to the requested stack. Additions are listed above with reasons.

## 2. Repository layout

```
scribewell/
├─ apps/desktop/
│  ├─ src/                       React app
│  │  ├─ app/                    routes, layout, providers
│  │  ├─ features/               one folder per feature: capture, transcript, editor,
│  │  │                          enhance, templates, chat, library (folders/search/people),
│  │  │                          calendar, sharing, integrations, settings, onboarding
│  │  ├─ components/             shared UI primitives
│  │  ├─ ipc/                    typed invoke/event wrappers + generated types (ts-rs)
│  │  ├─ i18n/                   locales
│  │  └─ styles/                 tokens, Tailwind entry
│  ├─ src-tauri/                 thin Tauri app crate: commands, events, tray, windows,
│  │                             capabilities, wiring of service crates
│  └─ e2e/                       Playwright + tauri-driver tests
├─ crates/
│  ├─ sw-core/                   domain types, AppError, ids, time, config paths
│  ├─ sw-db/                     schema, migrations, repositories, FTS indexing
│  ├─ sw-secrets/                keychain wrapper
│  ├─ sw-audio/                  capture traits, OS backends, file source, chunk writer,
│  │                             recovery, compaction, import
│  ├─ sw-stt/                    VAD, whisper engine, cloud adapters, model manager,
│  │                             echo suppression, streaming scheduler
│  ├─ sw-llm/                    provider trait + adapters, streaming, retries
│  ├─ sw-notes/                  note document model (ProseMirror JSON), authorship,
│  │                             prompt builder, enhance pipeline, templates, versions
│  ├─ sw-chat/                   retrieval (FTS5 + optional embeddings), citations, recipes
│  ├─ sw-calendar/               Google, Microsoft Graph, ICS; sync; reminders
│  ├─ sw-integrations/           Slack, Notion, webhook/Zapier, markdown sync, delivery queue
│  └─ sw-export/                 md/txt/html/json/zip export, backup/restore, share upload
├─ share-server/                 optional self-hosted share link server
├─ fixtures/                     sample meeting audio (generated by us), golden outputs
├─ docs/                         teardown, PRD, architecture, PROGRESS, ADRs
└─ .github/                      workflows, issue templates
```

### Module boundaries (enforced)
- Domain crates (`sw-*`) never depend on Tauri. They expose async services and emit events through a `tokio::broadcast` bus defined in `sw-core`.
- Only `src-tauri` knows about windows, IPC and plugins. Commands are one-liners that call a service and map `AppError`.
- The webview never sees secrets, raw SQL or file paths outside the app data dir.
- `sw-db` is the only crate that touches SQLite. Other crates use repository traits, so they can be tested with an in-memory DB.
- Dependency direction: `core ← db ← (audio, stt, llm) ← notes ← chat ← (calendar, integrations, export) ← app`. Cycles are a CI failure (`cargo-deny` + a workspace check).

## 3. Data model

All IDs are UUIDv7 text (sortable). Timestamps are UTC ms integers. Soft delete via `deleted_at`.

```sql
-- Meetings and capture
meetings(id PK, title, status CHECK IN ('draft','recording','processing','ready','failed'),
         scheduled_start, scheduled_end, started_at, ended_at, calendar_event_id NULL,
         template_id NULL, language NULL, summary_language NULL,
         created_at, updated_at, deleted_at NULL)
recordings(id PK, meeting_id FK, started_at, ended_at NULL,
           status CHECK IN ('active','finalized','recovered','failed'),
           mic_device NULL, loopback_device NULL, sample_rate, dir, error NULL)
audio_chunks(recording_id FK, channel CHECK IN ('me','them'), seq, path, start_ms,
             duration_ms, PRIMARY KEY(recording_id, channel, seq))
audio_files(recording_id FK, channel, path, codec, duration_ms, bytes)   -- after compaction

-- Transcript
transcript_segments(id PK, meeting_id FK, recording_id FK, channel, start_ms, end_ms, text,
                    avg_logprob NULL, engine, model, language, edited INTEGER, deleted_at NULL)
meeting_speakers(meeting_id FK, channel, display_name, person_id NULL, PRIMARY KEY(meeting_id, channel))

-- Notes: two documents per meeting, versioned
documents(id PK, meeting_id FK, kind CHECK IN ('raw','enhanced'), current_version_id,
          updated_at, UNIQUE(meeting_id, kind))
document_versions(id PK, document_id FK, content_json, plain_text, created_at,
                  cause CHECK IN ('typing','enhance','regenerate','template_switch','chat_rewrite','restore'),
                  template_id NULL, llm_run_id NULL)
note_blocks(document_id FK, block_id, position, block_type, origin CHECK IN ('user','ai','mixed'),
            text, PRIMARY KEY(document_id, block_id))          -- derived index of current version
block_citations(document_id FK, block_id, segment_id FK)        -- derived

-- Templates and recipes
templates(id PK, name, icon, description, is_builtin, spec_json, created_at, updated_at, deleted_at NULL)
recipes(id PK, name UNIQUE, prompt, scope CHECK IN ('single','multi'), model_override NULL, is_builtin)

-- Organization
folders(id PK, parent_id NULL, name, icon, color, description, position,
        default_template_id NULL, created_at, deleted_at NULL)
meeting_folders(meeting_id, folder_id, PRIMARY KEY(meeting_id, folder_id))
folder_rules(id PK, folder_id, kind CHECK IN ('recurring_series'), match_value)
tags(id PK, name UNIQUE, color); meeting_tags(meeting_id, tag_id, PRIMARY KEY(...))
companies(id PK, name, domain UNIQUE NULL, notes)
people(id PK, name, email UNIQUE NULL, company_id NULL, notes, merged_into NULL)
meeting_attendees(meeting_id, person_id, role, response, PRIMARY KEY(meeting_id, person_id))

-- Calendar
calendar_accounts(id PK, provider CHECK IN ('google','microsoft','ics'), label, config_json)
calendars(id PK, account_id FK, external_id, name, color, enabled)
calendar_events(id PK, calendar_id FK, external_id, series_id NULL, title, start_at, end_at,
                all_day, conference_url NULL, location NULL, attendees_json, my_response,
                status, etag, UNIQUE(calendar_id, external_id))

-- Chat
chats(id PK, scope_type CHECK IN ('meeting','folder','person','company','selection','all'),
      scope_ref_json, title, created_at, updated_at)
chat_messages(id PK, chat_id FK, role, content, citations_json, llm_run_id NULL, created_at)

-- Integrations
integrations(id PK, kind CHECK IN ('slack','notion','webhook','zapier','markdown_sync'),
             name, config_json, enabled)              -- secrets live in keychain, keyed by id
integration_rules(id PK, integration_id FK, trigger CHECK IN ('manual','enhanced','added_to_folder'),
                  folder_id NULL)
deliveries(id PK, integration_id, meeting_id, trigger, status, attempts, next_attempt_at,
           last_error NULL, created_at)

-- Misc
settings(key PK, value_json)
llm_runs(id PK, task, provider, model, input_tokens, output_tokens, started_at, duration_ms,
         status, error_code NULL)                     -- local diagnostics only, no content
embeddings(owner_type, owner_id, model, dim, vector BLOB, PRIMARY KEY(owner_type, owner_id, model))
search_index USING fts5(meeting_id UNINDEXED, kind UNINDEXED, ref_id UNINDEXED, title, body,
                        tokenize='unicode61 remove_diacritics 2')
schema_migrations: tracked with PRAGMA user_version
```

**Authorship model.** The enhanced document is ProseMirror JSON. Every block node has `attrs.origin` (`user` | `ai`) and `attrs.cites` (segment IDs). Inline `userText` marks flag user-typed spans inside AI blocks. When the user types into an AI block, the editor wraps the new text in `userText`, and the block becomes `mixed`. `note_blocks` and `block_citations` are rebuilt from the JSON on every save, which keeps search and provenance fast without making JSON the query surface.

**Migrations.** Numbered SQL files embedded with `include_str!`, applied in one transaction each, and recorded in `user_version`. Before the first pending migration, `VACUUM INTO backups/pre-v{N}.db`. On failure the app rolls back, restores the backup, and shows the error with the log path.

## 4. Capture and transcription pipeline

```
 mic (cpal) ──────┐                         ┌─► chunk writer: 5 s WAV files, fsync, DB row ─► on stop: FLAC compaction
                  ├─► per-channel ring buf ─┤
 loopback ────────┘   (native rate)         └─► resample 16 kHz mono ─► VAD ─► utterance queue
                                                                                   │
                         echo suppressor ◄── STT worker (one whisper ctx, GPU) ◄───┘
                               │
                               └─► transcript_segments + live event to UI
```

- **RecordingSession** is a tokio actor per active meeting that owns the streams. Commands are start, pause, resume, stop, set device. It emits level meters at 20 Hz, state changes and errors.
- **Crash safety.** Each channel is written as consecutive self-contained 5-second WAV chunks. Each chunk file is fsynced, then its row is inserted (WAL mode). The `recordings.status='active'` row is the marker. On startup, any `active` recording without a live session is a crash survivor: we validate the chunks, mark the recording `recovered`, and queue transcription for the untranscribed tail. Worst case, about 5 s of audio is lost.
- **Compaction.** After stop, chunks are concatenated per channel into FLAC (about half the WAV size, lossless, pure Rust) and then deleted. Retention policy runs daily.
- **Windows loopback.** Opening the default render device as an input stream in cpal's WASAPI host gives shared-mode loopback. When the output device changes mid-call, we reopen the stream.
- **Echo suppression.** If a mic utterance overlaps a system utterance in time by more than 50 % and the normalized text similarity is at least 0.8, we drop the mic one. Headphones remain the recommendation, shown in the UI when echo is detected.
- **STT scheduling.** Utterances are capped at 30 s. Mic and system are interleaved by start time. The model stays loaded while recording. Context: the last ~200 characters per channel plus the custom vocabulary go into the whisper initial prompt.
- **Model manager.** A model registry (name, URL on the public Hugging Face whisper.cpp repo, SHA-256, size) is checked in. Downloads support HTTP range resume, verify the checksum, and are atomic via temp file + rename.
- **Cloud STT.** The same utterance queue feeds an adapter that uploads WAV per utterance (OpenAI/Groq) or streams (Deepgram).
- **Imports and tests.** A `FileSource` implements the same `CaptureSource` trait. Imports and the integration test use it.

## 5. Enhancement pipeline

1. **Gather** raw notes (lines with stable indices), transcript segments (`[S:id] Me/Them(name) mm:ss text`), meeting metadata, profile context, template spec, output language.
2. **Budget.** Estimate tokens. If the transcript exceeds the model budget, run **map** passes over transcript windows that produce cited fact lists, then a **reduce** pass with notes plus facts.
3. **Prompt.** Built by a pure function in `sw-notes::prompt`, snapshot-tested with insta. The output protocol is line-based markdown with trailing tags, which streams well and is robust for small local models:
   `- Moving to annual billing in Q3 {u:2} {c:41,42}`
   `{u:n}` means "derived from user line n" and `{c:…}` lists the segments cited.
4. **Stream parse.** Complete lines are parsed incrementally into blocks and sent to the UI over a Channel. The UI renders them with the "Enhancing notes" state. Cancel aborts the request.
5. **Post-process (deterministic):**
   - **Preservation guarantee.** Each raw line must be covered by a `{u:n}` block whose text, normalized, has similarity of at least 0.75 to the original. Missing lines are inserted verbatim as `origin=user` under the section with the best BM25 match, or under a final "Your notes" section.
   - **Citation validation.** Unknown segment IDs are dropped.
   - User-derived blocks get `origin=user`, everything else `origin=ai`.
6. **Persist** as a new `document_versions` row (cause `enhance` / `regenerate` / `template_switch`) and record an `llm_runs` row. Undo is one click back to the previous version. The editor's own undo stack covers edits.
7. **Regenerate guard.** If the current enhanced version has user edits since the last generation, confirm before overwriting. The old version is still kept.

## 6. Chat

- **Retrieval.** The question becomes a sanitized FTS5 query over `search_index`, filtered to the scope's meeting IDs. We take the top-k segments plus ±1 neighbor and the top note blocks. If embeddings are enabled (Ollama embedding model), we merge in vector top-k using reciprocal rank fusion. For a single meeting that fits the context window, we skip retrieval and send the whole transcript.
- **Answering.** The model must cite as `[S:id]` or `[N:block]`. The renderer validates citations against the retrieved set and turns them into chips that jump to the timestamp or the note.
- **Recipes** are stored prompts injected as the user turn. Scope type controls where they appear.
- **Rewrite** is a chat intent ("edit the note") that runs the enhance pipeline in "revise" mode with the instruction and produces a new version.

## 7. LLM provider layer

```rust
trait LlmProvider {
    fn id(&self) -> ProviderId;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError>;
    async fn stream_chat(&self, req: ChatRequest, sink: TokenSink) -> Result<Completion, AppError>;
}
```
- Adapters: Anthropic Messages (SSE), OpenAI Chat Completions (SSE; OpenRouter uses the same adapter with a different base URL and headers), Gemini `streamGenerateContent` (SSE), Ollama `/api/chat` (NDJSON). Each normalizes deltas, usage and stop reason, and maps HTTP errors onto `llm.auth`, `llm.rate_limited`, `llm.context_exceeded`, `llm.network` and `llm.server`.
- Retries: exponential backoff with jitter for 429/5xx/network (max 3), never after the first token has streamed.
- Default model IDs are configurable and listed per provider via `list_models` where the API allows it.
- Unit tests run each adapter against wiremock fixtures of real stream formats: happy path, split chunks, error bodies, malformed lines.

## 8. Errors and logging

- **`AppError`** (thiserror) has a stable `code` (e.g. `audio.permission_denied`, `audio.no_device`, `audio.loopback_unavailable`, `stt.model_missing`, `stt.download_failed`, `stt.checksum_mismatch`, `llm.missing_key`, `llm.auth`, `llm.rate_limited`, `llm.network`, `db.migration_failed`, `io.disk_full`, `calendar.auth_expired`, `integration.delivery_failed`), a human `message`, optional `detail`, a `retryable` flag and an optional `action` (e.g. `open_settings:microphone`). It serializes to a TS discriminated union. The UI has one mapping table from code to copy and action, so there are no generic "something went wrong" toasts.
- No `unwrap()`/`expect()` outside tests and provably infallible spots (clippy `unwrap_used = deny` in library crates). Panics in background tasks are caught, logged and surfaced as `internal` errors. A panic hook writes a crash marker for the next launch.
- **Logging.** `tracing` with spans per meeting, recording and LLM run. Daily rolling JSON log files in the OS app log dir, 7 days kept. A redaction layer masks anything that looks like an API key or bearer token. Note and transcript content is never logged at info level or above. Frontend errors are forwarded through a `log` command. Settings has "Open logs folder" and "Copy diagnostics" (versions, OS, devices, settings minus secrets).
- **No telemetry.** The only optional outbound call that isn't user-configured is the update check, which is off by default.

## 9. Testing strategy

| Level | What | Tooling |
|---|---|---|
| Unit (Rust) | migrations up from empty and from every prior version; repositories; FTS indexing; authorship rebuild; prompt builder (snapshots); output parser; preservation and citation post-processing; echo suppressor; VAD segmentation on synthetic signals; provider adapters on recorded stream fixtures; retry policy; webhook signing; markdown/HTML export | cargo test, insta, wiremock, tempfile |
| Unit (TS) | editor authorship plugin (typing in AI block → userText), stores, IPC error mapping, formatters | vitest, Testing Library |
| Integration | **record → transcribe → enhance**: `FileSource` plays `fixtures/audio/meeting-2ch.wav` (scripted two-voice meeting we generate with offline TTS, left = me, right = them) through the real session, chunk writer, VAD and whisper `tiny.en`; asserts segments on both channels, keyword recall ≥ 80 %, crash-recovery path (kill mid-stream, recover, resume); enhancement via a deterministic mock provider and, when `SW_TEST_OLLAMA=1`, a real Ollama model, asserting every user line is preserved and every citation is valid | cargo test (`--features integration`), model cached in CI |
| UI E2E | onboarding, create meeting, type notes, run enhance with mocked IPC, template switch, undo, search, chat citations | Playwright against Vite build with `@tauri-apps/api/mocks` |
| App smoke | launch the real app, create a note, quit, relaunch, note persists | tauri-driver (WebDriver) on Windows in CI |
| Manual verification per milestone | Exercising the real feature on this Windows machine (real mic, real loopback with audio playing, real models), recorded in PROGRESS.md with what was and wasn't exercised | — |

**Quality gate script** (`npm run verify`): `tsc --noEmit`, `eslint --max-warnings 0`, `prettier --check`, `vitest run`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tauri build`. CI runs the same.

## 10. Platform notes

- **Windows (first):** WASAPI via cpal; mic privacy denial detected via the stream error plus the consent-store registry key, with a deep link to `ms-settings:privacy-microphone`. Mic-in-use detection (FR-9.6) polls the CapabilityAccessManager consent store for apps with an open microphone session. Installer: NSIS + MSI.
- **macOS:** `NSMicrophoneUsageDescription` and `NSAudioCaptureUsageDescription`; process tap on 14.2+, ScreenCaptureKit on 13–14.1. Needs signing and notarization for a smooth install (requires your Apple Developer account).
- **Linux:** PulseAudio/PipeWire monitor source; Secret Service required for keys; AppImage + .deb + .rpm.
- **Honest limit:** this machine is Windows, so I can exercise Windows capture for real. macOS and Linux backends will be compiled and unit-tested in CI and manually testable by you or contributors, but I can't exercise their capture paths myself.

## 11. Milestones and acceptance criteria

Each milestone ends with: the quality gate green, the feature exercised in the running app on Windows, `docs/PROGRESS.md` updated (what works, what was tested, known issues), and a commit.

**M0 Environment and scaffolding**
- Toolchain installed (see §12). Monorepo, Cargo workspace, Vite/React/Tailwind app, Tauri 2 shell opens a window. Lint/format/test/verify scripts. CI workflow runs the gate on Windows, macOS and Linux. MIT LICENSE.
- *Accept:* `npm run verify` passes locally and in CI; `tauri dev` shows the shell.

**M1 Foundation**
- Schema v1 + migration runner + backups; settings service; keychain service; app shell (sidebar, routes, theme); onboarding with permission flow (mic check, loopback test tone, model/key choice, Ollama detection, profile).
- *Accept:* fresh install walks through onboarding; denying mic shows the banner with a working settings link; a key saved in onboarding appears in Windows Credential Manager and not in any file under the app dir (grep verified); restart keeps settings; migration tests pass.

**M2 Capture**
- Mic + loopback capture, device pickers, hot-plug fallback, level meters, tray and in-app indicator, pause/resume/stop, chunk writer, crash recovery, FLAC compaction, retention, audio import.
- *Accept:* record 5 minutes with YouTube audio playing and me speaking, and both meters move; files present per channel; killing the process with Task Manager mid-recording, then relaunching, gives a recovery prompt and the audio up to ≤ 5 s before the kill is playable; unplugging a USB mic falls back without stopping.

**M3 Transcription**
- Model manager (download, resume, verify, delete), whisper engine with Vulkan/CPU, VAD streaming, live transcript panel with Me/Them bubbles, rename speaker, echo suppression, language and vocabulary, cloud STT adapters, transcript search/copy/edit/delete/play-from-here, re-transcribe.
- *Accept:* live transcript appears within ~5 s of speech ending on `base` with GPU; the integration test passes; killing the network mid-download and restoring it resumes the download; a corrupted model file is detected; a missing Deepgram key shows a precise error.

**M4 Notepad**
- TipTap editor with markdown rules, slash menu, images, checkboxes; debounced autosave plus crash-safe flush; raw vs enhanced tabs; authorship styling.
- *Accept:* typing at full speed shows no lag (profiled); force-kill within 1 s of typing loses ≤ 500 ms of text; user text vs AI text visibly distinct in both themes and announced to screen readers.

**M5 Enhance**
- Provider layer (5 providers), prompt builder, streaming parser, preservation and citation post-processing, Enhancing state with cancel, auto-enhance on stop, regenerate with guard, version history with undo/restore, map-reduce for long meetings, summary language, auto title.
- *Accept:* with Ollama locally and with at least one cloud key (if you provide one), a real meeting enhances end to end; in a 20-meeting synthetic corpus, 100 % of user lines are preserved; clicking a citation jumps to the right moment; undo restores the previous version; no network plus a cloud provider shows a queued/retry state.

**M6 Templates**
- 12+ built-in templates, editor with sections and live preview, per-meeting switch (including slash in the editor), defaults per global/folder/series, import/export.
- *Accept:* switching a finished meeting from Auto to Candidate interview regenerates in that structure; a custom template round-trips through export/import.

**M7 Chat**
- Scoped chat (meeting incl. live, folder, person, company, selection, all), FTS5 retrieval + optional embeddings, validated citations, recipes with `/`, rewrite-as-version, persisted threads, file attachments.
- *Accept:* asking a question whose answer is in a 2-week-old meeting returns it with a citation that opens that meeting at the right timestamp; a recipe runs on a folder; "make next steps more detailed" creates an undoable version.

**M8 Organization**
- Folders (nested, multi-membership, drag-drop, bulk), search with snippets and filters, people/companies auto-detection with edit/merge, tags, trash with purge, @mentions, auto-file recurring meetings.
- *Accept:* with 1,000 generated meetings, search returns in < 100 ms; attendees from a calendar event create people and companies (free-mail domains excluded); merging two people merges their meetings; trash restore works.

**M9 Calendar**
- Google and Microsoft OAuth with your client credentials (guided setup), ICS subscriptions, calendar selection, coming-up list, reminders with Start notes / Open link, optional auto-start, pre-meeting brief, Windows mic-in-use detection prompt.
- *Accept:* with your test calendar, an event 2 minutes out triggers a reminder, and clicking it opens a note linked to the event with attendees populated; token expiry triggers a re-auth prompt instead of failing silently; an ICS URL works with no OAuth. *(Needs your Google/Microsoft OAuth client IDs; ICS can be verified without them.)*

**M10 Sharing and export**
- Copy as md/plain/HTML; export md/txt/html/pdf/json; bulk zip; backup/restore; follow-up email draft via mailto; share server + publish/revoke.
- *Accept:* pasted HTML keeps formatting in Google Docs and Outlook; backup from one profile restores into a clean profile with identical content; a share link served by the local share-server Docker image renders read-only and stops working after revoke.

**M11 Integrations**
- Slack webhook (manual + per-folder auto), Notion (token + chosen database), generic webhook with HMAC + retries + delivery log, Zapier-compatible mode, markdown folder sync.
- *Accept:* each integration delivers to a real endpoint (Slack/Notion need your test workspace; webhook verified against a local receiver checking the signature); a failing endpoint retries and shows the error in the delivery log; the Obsidian vault folder updates within seconds of an edit.

**M12 Polish**
- Shortcut map + global shortcut, command palette, focus mode, themes, empty states, motion with reduced-motion support, accessibility pass (axe in Playwright + manual keyboard/screen-reader check with Narrator), i18n extraction + pseudo-locale, consent message helper.
- *Accept:* axe reports 0 serious/critical violations on all main screens; every action reachable by keyboard; pseudo-locale shows no hard-coded strings.

**Phase 4 Ship**
- README (demo GIF, install per OS, BYOK, how it works), CONTRIBUTING, issue templates, release workflow producing installers for all three OSes, landing page copy, launch post, REA audit of our own built installer (no secrets bundled, no unexpected network endpoints, expected permissions only), final audit against the teardown matrix.

## 12. Environment prerequisites and blockers

Measured on this machine on 2026-10-18:

| Item | Status | Needed for |
|---|---|---|
| Node 24.15, npm 11.4, git 2.48, Python 3.13, winget | Present | — |
| WebView2 runtime 155 | Present | Tauri |
| Ollama 0.40.3 | Installed, not running | Local LLM |
| NVIDIA RTX 3050 6 GB | Present | GPU whisper via Vulkan |
| Rust (rustup, stable MSVC toolchain) | **Missing** | Everything Rust |
| Visual Studio 2022 Build Tools (C++ workload, Windows SDK) | **Missing** | Rust MSVC linker, whisper.cpp |
| CMake | **Missing** | whisper.cpp build |
| LLVM/Clang (libclang) | **Missing** | whisper-rs bindgen |
| Vulkan SDK | **Missing** | GPU whisper build (CPU build works without it) |
| **Free disk on C:** | **15 GB of 475 GB (97 % full)** | Toolchain ≈ 10–12 GB, build caches 5–10 GB, models 0.1–3 GB |
| Project location | Inside OneDrive sync | Build dirs would churn OneDrive; recommend `C:\dev\scribewell` (excluded from sync) |
| GitHub repo + `gh` CLI | `gh` missing, no remote | CI and releases (Phase 4) |
| Google / Microsoft OAuth client IDs | You provide (M9) | Calendar sync |
| Slack webhook, Notion token, test cloud LLM/STT keys | You provide, optional | Verifying those adapters for real |
| Code-signing certs (Windows Authenticode, Apple Developer ID) | You decide (Phase 4) | Installers without SmartScreen/Gatekeeper warnings |
