# Scribewell: product requirements

Status: draft for approval. Working name **Scribewell** (see "Name" below).

## 1. Summary

Scribewell is an open-source (MIT), local-first AI notepad for meetings. You jot rough notes while a call runs; Scribewell transcribes your microphone and your computer's audio on-device, without a meeting bot, and when the call ends it turns your notes plus the transcript into clean, structured notes. Your own words stay visibly yours. Everything lives in a local SQLite database. AI runs locally (whisper.cpp + Ollama) or through the user's own API keys. There are no accounts, no paywalls, no usage caps and no telemetry.

## 2. Goals

1. Match the core Granola experience on desktop: bot-free capture, live transcript, rough notes → enhanced notes, templates, chat with citations, organization, sharing, integrations.
2. Ship every "paid tier" capability that makes sense for a single-user local app, for free: unlimited history, model choice, integrations, briefs, follow-up drafts, export.
3. Work fully offline when the user chooses local models.
4. Never lose a meeting: crash-safe audio buffering and recovery.
5. Windows first, then macOS, then Linux.

## 3. Non-goals (v1)

- Mobile and watch apps.
- Multi-user workspaces, cloud sync, SSO/SCIM, admin consoles, compliance certifications.
- Joining calls as a bot, or scraping meeting apps' UI for participant names (deferred, see teardown).
- Enriching people with data from the web.
- Any server operated by the project. The optional share server is self-hosted by the user.

## 4. Users and primary scenarios

- **Individual professional** (PM, founder, recruiter, sales, consultant) on back-to-back calls who wants good notes without typing everything.
- **Privacy-sensitive user** (legal, health, finance, security, journalists) who cannot send audio to a vendor.
- **Tinkerer** who wants notes in Obsidian/Notion/Slack and models of their choice.

Scenarios:
- S1. A calendar meeting starts; a notification offers "Start notes"; user clicks, types 6 rough lines, ends the call, gets structured notes in their chosen template within ~30 s (cloud LLM) or a couple of minutes (local LLM, hardware dependent).
- S2. An unscheduled call: user hits the global shortcut, a note opens and recording starts.
- S3. The app crashes or the laptop dies mid-meeting. On relaunch, Scribewell finds the partial recording and offers to recover, transcribe and enhance it.
- S4. A week later the user asks "what did Dana say about the pricing change?" across all notes and gets an answer citing transcript timestamps they can click.
- S5. After an interview, the user switches the template from "Auto" to "Candidate interview", regenerates, and undoes if they prefer the first version.
- S6. Notes from the "Customers" folder auto-post to a Slack channel and mirror into an Obsidian vault.

## 5. Functional requirements

IDs map to milestones in `architecture.md`. "Must" = required for v1.0.

### 5.1 Foundation (M1)
- FR-1.1 Must: desktop app shell with sidebar (home, folders, people, companies, search, trash, settings), resizable panes, light/dark/system theme.
- FR-1.2 Must: local SQLite database with versioned, forward-only migrations applied at startup; automatic pre-migration backup.
- FR-1.3 Must: settings stored in DB (non-secret) and OS keychain (secrets: API keys, OAuth tokens, webhook secrets). Secrets never written to disk in plain text, never sent to the webview after entry (write-only from the UI; UI sees only "configured / not configured" and last 4 chars).
- FR-1.4 Must: onboarding: welcome → privacy explainer → microphone permission check (with deep link to OS privacy settings when denied) → system-audio check (play a test tone, verify loopback captures it) → transcription engine choice (download a local model with progress, or enter a cloud STT key) → LLM choice (detect Ollama, or enter a key) → optional profile context (role, company, preferences) → optional calendar → done. Every step skippable and revisitable from Settings.

### 5.2 Capture (M2)
- FR-2.1 Must: capture microphone and system output simultaneously as two separate channels.
- FR-2.2 Must: device picker for input (with "System default") and output-loopback source; remembered per machine; hot-plug handling (device disappears → fall back to default, surface a non-blocking warning).
- FR-2.3 Must: live level meters for both channels; persistent recording indicator (in-app header plus system tray icon state); elapsed time.
- FR-2.4 Must: crash-safe buffering: audio flushed to disk in short self-contained chunks so at most a few seconds are lost if the process dies; startup recovery of unfinished recordings.
- FR-2.5 Must: pause/resume and stop; "resume" on an ended note appends a new capture session.
- FR-2.6 Must: audio kept locally (default) with retention options: keep forever, delete after N days, delete after enhancement. Stored compressed after the meeting ends.
- FR-2.7 Must: import an audio file (wav, mp3, m4a, flac, ogg) as a meeting.
- FR-2.8 Should: auto-stop after configurable silence (default 15 min).

### 5.3 Transcription (M3)
- FR-3.1 Must: local whisper.cpp engine with model picker (tiny, base, small, medium, large-v3, large-v3-turbo; English-only variants where they exist), size and expected speed shown, download with progress, resume, checksum verification, cancel, delete.
- FR-3.2 Must: GPU acceleration when available (Vulkan on Windows/Linux, Metal on macOS), automatic CPU fallback.
- FR-3.3 Must: streaming: VAD splits each channel into utterances; utterances are transcribed as they complete; the live panel updates within a few seconds of speech ending (target ≤ 5 s on base model with GPU).
- FR-3.4 Must: speaker labels "Me" (mic) and "Them" (system); user can rename "Them" per meeting (e.g. "Dana"); renames apply to that meeting's transcript display and to prompts.
- FR-3.5 Must: echo handling when the user is on speakers: mic utterances that duplicate simultaneous system-audio speech are suppressed.
- FR-3.6 Must: language: auto-detect or a fixed language per meeting, default in settings; translate-to-English option.
- FR-3.7 Must: custom vocabulary list used as a recognition hint.
- FR-3.8 Must: BYOK cloud STT adapters: Deepgram, OpenAI, Groq (chunked upload per utterance; Deepgram streaming optional).
- FR-3.9 Must: transcript panel: search, copy all, copy/delete/edit a segment, jump to timestamp, play the audio at that timestamp.
- FR-3.10 Must: re-transcribe a meeting with a different model or engine (possible because audio is kept).

### 5.4 Notepad (M4)
- FR-4.1 Must: fast rich-text editor: headings, bullets, numbered lists, checkboxes, bold/italic/underline/strike, inline code, links, quotes, images (paste/drop, stored locally), markdown shortcuts, slash menu.
- FR-4.2 Must: auto-save (debounced ≤ 500 ms after typing stops, plus on blur/close); no data loss on crash beyond the debounce window.
- FR-4.3 Must: authorship: every piece of text knows whether the user typed it or the AI wrote it. User text and AI text are visually distinct in all views (our own styling: AI text slightly muted with a thin accent rule on AI blocks; user text full contrast). If the user edits AI text, the edited spans become user-authored.
- FR-4.4 Must: two tabs per meeting: "My notes" (raw, exactly as typed) and "Enhanced". The raw notes are never modified by AI.

### 5.5 Enhance (M5)
- FR-5.1 Must: enhancement runs automatically when recording stops (configurable) and on demand.
- FR-5.2 Must: inputs: raw notes, transcript (with speaker labels and timestamps), meeting metadata (title, time, attendees), user profile context, selected template.
- FR-5.3 Must: guaranteed preservation: every non-empty line of the user's raw notes appears in the enhanced notes, verbatim or with only typo fixes, marked as user-authored. If the model drops one, the app re-inserts it deterministically in the most relevant section.
- FR-5.4 Must: streaming output renders block by block with a visible "Enhancing notes" state and a cancel button.
- FR-5.5 Must: every AI block carries citations to transcript segments; hovering/clicking shows the source excerpt and can play the audio there.
- FR-5.6 Must: regenerate (same template), regenerate with another template, warning before overwriting manual edits.
- FR-5.7 Must: undo/redo plus version history for enhanced notes; restore any previous version.
- FR-5.8 Must: providers: Ollama, Anthropic, OpenAI, Gemini, OpenRouter; per-task model choice (enhance, chat, titles).
- FR-5.9 Must: output language: same as meeting, or a fixed language.
- FR-5.10 Must: long meetings: transcripts exceeding the model's context are processed with a map-reduce strategy.
- FR-5.11 Should: auto-generate a meeting title for ad-hoc meetings.

### 5.6 Templates (M6)
- FR-6.1 Must: at least 12 built-in templates written for this project: Auto (general), 1:1, Team standup, Sales discovery call, Customer success check-in, Candidate interview, User research interview, Project kickoff, Weekly team sync, Brainstorm, Board/investor update, Retrospective, Lecture/class notes.
- FR-6.2 Must: custom template editor: name, icon, description/when to use, instructions, ordered sections (title + guidance each), output style options (length, tone, bullets vs prose), live preview against a sample meeting.
- FR-6.3 Must: per-meeting template switch (also mid-meeting via slash menu); default template globally, per folder and per recurring event series.
- FR-6.4 Must: duplicate, edit, delete custom templates; built-ins are read-only but can be duplicated; import/export as files.

### 5.7 Chat (M7)
- FR-7.1 Must: chat scopes: current meeting (also live during recording), a folder, a person, a company, a selection of meetings, all notes. Scope shown and changeable in the chat header.
- FR-7.2 Must: answers cite transcript moments and notes as clickable citations; citations are validated against the retrieved context (no invented IDs rendered).
- FR-7.3 Must: retrieval: SQLite FTS5 (BM25) with scope filters; optional local embeddings (Ollama) for semantic recall; hybrid ranking.
- FR-7.4 Must: recipes: saved prompts with name, scope type and optional model, invoked with `/`; built-in recipes included (e.g. "What did I miss?", "Action items by owner", "Draft follow-up email", "Objections raised", "Decisions log", "Weekly recap").
- FR-7.5 Must: rewrite enhanced notes via chat ("make next steps more detailed") as a new note version (undoable).
- FR-7.6 Must: chat threads persisted per scope, deletable.
- FR-7.7 Should: attach local files (txt, md, pdf text) as extra context.

### 5.8 Organization (M8)
- FR-8.1 Must: folders with nesting, icon, color, description; a note can be in several folders; drag-and-drop; bulk select.
- FR-8.2 Must: full-text search across titles, raw notes, enhanced notes and transcripts with highlighted snippets; filters (date range, folder, person, company, tag); results in < 100 ms for 1,000 meetings.
- FR-8.3 Must: people and companies auto-created from calendar attendees (name + email; company from email domain, ignoring free-mail domains); editable, mergeable, manually addable; person/company pages list meetings and offer chat.
- FR-8.4 Must: tags (free-form, colored), filter by tag.
- FR-8.5 Must: trash with restore; auto-purge after 30 days (configurable).
- FR-8.6 Must: @mentions of people in notes.
- FR-8.7 Should: auto-file recurring meetings into the folder of a previous instance.

### 5.9 Calendar (M9)
- FR-9.1 Must: Google Calendar and Microsoft 365/Outlook via OAuth 2.0 + PKCE using the user's own client ID (and secret where the provider requires one for desktop apps), with a step-by-step in-app guide. Read-only scopes.
- FR-9.2 Must: ICS URL subscription as a no-OAuth alternative (also covers Apple/iCloud shared calendars).
- FR-9.3 Must: "Coming up" list on home; choose calendars to include; hide declined and all-day / non-meeting blocks.
- FR-9.4 Must: reminder notification N minutes before (default 1) with "Start notes" and "Open call link" actions; optional auto-start recording at start time (off by default).
- FR-9.5 Must: pre-meeting brief: when opening an upcoming meeting, a short local summary of previous meetings with the same attendees/company and open action items.
- FR-9.6 Should (Windows first): detect when another app starts using the microphone and offer to start notes.

### 5.10 Sharing and export (M10)
- FR-10.1 Must: copy enhanced notes as Markdown, plain text or rich HTML (pastes well into Docs, Slack, email); copy transcript.
- FR-10.2 Must: export a note to .md, .txt, .html, .pdf (via print), .json; bulk export of a folder or everything as a zip; full backup and restore (JSON + attachments + optional audio).
- FR-10.3 Must: follow-up email draft generated from a meeting, opened in the default mail client (mailto) or copied.
- FR-10.4 Should: optional share link: publish a static, read-only HTML rendering of a note to a self-hosted Scribewell share server (provided in this repo, single binary / Docker); revoke link; off by default; transcript excluded unless explicitly included.

### 5.11 Integrations (M11)
- FR-11.1 Must: Slack via incoming webhook: manual "Send to Slack" and per-folder auto-post on enhancement.
- FR-11.2 Must: Notion via the user's internal integration token: create a page in a chosen database (title, date, attendees, tags, body blocks).
- FR-11.3 Must: generic webhook (JSON payload, HMAC-SHA256 signature header, retries with backoff, delivery log); Zapier/Make/n8n-compatible "catch hook" mode.
- FR-11.4 Must: markdown folder sync: mirror every note as `.md` with YAML front-matter into a chosen folder (Obsidian friendly), with configurable filename pattern and one-way overwrite semantics.
- FR-11.5 Must: per-integration triggers: manual, on enhancement complete, on add to folder.

### 5.12 Polish (M12)
- FR-12.1 Must: keyboard shortcuts for all primary actions, a global shortcut to start/stop recording, and a command palette (Ctrl/Cmd+K).
- FR-12.2 Must: dark/light/system theme; focus mode (chrome fades while typing).
- FR-12.3 Must: designed empty states for home, folder, search, people, chat, transcript.
- FR-12.4 Must: motion: enhancement streaming, panel transitions; all respect `prefers-reduced-motion`.
- FR-12.5 Must: accessibility: full keyboard operation, visible focus, ARIA roles, WCAG 2.1 AA contrast in both themes, screen-reader announcements for recording state and enhancement completion.
- FR-12.6 Must: i18n scaffolding: all UI strings externalized; English shipped; pseudo-locale for testing.
- FR-12.7 Should: "consent message" helper: copy a short notice to paste into the meeting chat.

## 6. Non-functional requirements

- **Privacy:** no network calls except those the user configured (model downloads from a listed source, chosen AI providers, calendar, integrations, update check if enabled). A "Network activity" page lists every outbound destination used. Update checks are opt-in.
- **Security:** secrets in OS keychain; CSP locked to `self`; Tauri capabilities least-privilege; no remote code; webhooks signed; share server requires a token.
- **Performance:** cold start < 2 s on a mid-range laptop; editor keystroke latency < 16 ms; idle CPU < 1 %; recording + live base-model transcription < 25 % CPU on a 4-core laptop with GPU offload.
- **Reliability:** no data loss across crash, kill or power loss beyond 5 s of audio and 500 ms of typing. Migrations are transactional and backed up.
- **Portability:** Windows 10 1903+ / 11 (x64), macOS 13+ (Apple Silicon and Intel), Ubuntu 22.04+ / Fedora 39+ (x64).
- **Licensing:** MIT for all project code; all dependencies MIT/Apache/BSD/ISC/Zlib-compatible; whisper models (MIT) downloaded at runtime from their public host, not bundled.

## 7. Failure paths (must be handled with clear UI, no crashes)

| Failure | Behavior |
|---|---|
| Microphone permission denied | Banner with reason and button to open OS privacy settings; recording of system audio alone still possible. |
| No input device / device removed mid-call | Fall back to default; if none, continue system-only and warn. |
| System loopback unavailable | Continue mic-only; explain; link to troubleshooting. |
| Model not downloaded | Offer download or switch to cloud STT; audio still recorded so it can be transcribed later. |
| Model download fails / checksum mismatch | Retry with resume, show error and the URL; delete partial file on checksum failure. |
| No network | Local engines work; cloud steps queue and show "waiting for network" with retry; nothing is lost. |
| Missing or invalid API key | Inline error naming the provider and setting; offer local fallback. |
| Provider rate limit / 5xx | Exponential backoff with jitter (max 3), then surface error with "retry". |
| App killed mid-recording | Recovery dialog on next launch; recovered audio transcribed and attached to the original note. |
| Disk full | Stop recording gracefully, keep what was written, warn. |
| DB migration failure | Abort startup, restore backup, show error with log path. |

## 8. Success metrics (local, no telemetry)

Measured by us in testing, not collected from users: S1–S6 pass end to end; enhancement keeps 100 % of user lines in the test corpus; WER of local base.en on the fixture ≤ the published whisper figures + 5 pts; zero data loss in kill-during-recording tests.

## 9. Name

**Scribewell** (scribe + well, as in "write it down well"). Checked on 2026-10-18:
- Web search: no app, product or company found with that name.
- `scribewell.app` and `scribewell.dev`: no DNS records (likely unregistered; registrar check still needed). `scribewell.com` resolves (taken).
- npm package `scribewell`: available.
- GitHub: two personal repos with no stars; no organization claimed in our checks.
- Not checked: formal trademark databases (USPTO/EUIPO). Recommended before public launch.

Alternate if you prefer: **Hushpad** (`.app`/`.dev` also unresolved, npm free, one tiny GitHub repo), though "hush" may read as covert recording, which is the wrong signal for a product that must encourage consent.

The current folder name "Cranola" should not be used publicly; it is one letter from Granola's trademark.
