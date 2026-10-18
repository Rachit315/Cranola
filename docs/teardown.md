# Granola product teardown (clean-room)

Date of research: 2026-10-18.

## Method and boundaries

- Sources were limited to Granola's public marketing site, public pricing page, public help center (docs.granola.ai), public changelog (granola.ai/updates) and third-party reviews or press. Everything below is a paraphrase of observed public behavior. No Granola text, code, asset, icon or font is reproduced or used.
- Granola's app binary, its network traffic, its APIs and its MCP endpoint were **not** inspected, called or decompiled. No account was created and no sign-in was attempted. REA (the reverse-engineering MCP) is installed in this environment but is reserved for auditing **our own** builds.
- Anything the public sources do not establish is marked **UNVERIFIED**. Where a detail is inferred rather than stated, it says so.
- Help-center pages describe the product as of their last edit. Some pages disagree with each other (for example template-menu labels); the more recent wording is used and the conflict noted.

---

## 1. Core loop

1. **Before the meeting.** The home screen shows upcoming calendar events ("coming up", 5 at a time, pageable). Clicking one creates a note tied to that event. You can open it early to jot prep notes. One minute before a calendar event with two or more attendees, Granola shows its own in-app pop-up (top-right of the screen, not an OS notification). Clicking it opens the call URL and the matching note.
2. **During the meeting.** The note opens as a blank editor. Transcription starts when the note is opened at or after the scheduled start time, from the notification, or from "New note" for ad-hoc calls. It never starts without a note being open. By default the window repositions itself next to the call (toggleable). A live-meeting indicator follows you when you switch to other apps. A collapsible transcript panel shows live text. The user types terse, messy notes; typos and abbreviations are expected.
3. **Meeting end.** Granola stops when it detects the end: the call app releases the mic, the scheduled end time passes, 15 minutes pass with no new audio, or the machine sleeps. Manual stop is always available, and "Resume" appends more audio to the same note.
4. **Enhance.** Granola then automatically generates "enhanced notes" from three inputs: the transcript, the user's raw notes and the calendar event metadata (title, attendees, time). The user's own lines are treated as the important points to expand on.
5. **After.** The user edits the enhanced notes directly, switches template (regenerates), regenerates with the same template, asks chat to rewrite sections, shares, or pushes to an integration.

The essential idea worth matching: **the human writes the skeleton, the AI fills in the flesh, and the human's words stay visibly theirs.**

## 2. Audio capture without a bot

- Capture happens on the local machine: microphone plus the combined system output stream. Nothing joins the call.
- **System audio is the whole output mix.** Per-app isolation is not supported, so music or other sounds playing on the machine end up in the transcript.
- **macOS:** requires macOS 14+. Two permissions requested during onboarding: Microphone, and Screen & System Audio Recording. The exact API (ScreenCaptureKit vs Core Audio process tap) is **UNVERIFIED**. On macOS < 14.2, non-admin users may miss the call-ended signal (suggests the end-of-call detection relies on an OS-level audio-in-use signal; inferred).
- **Windows:** No OS permission prompts; relies on Windows microphone privacy settings ("let desktop apps use microphone"). Mechanism (WASAPI loopback or other) is **UNVERIFIED**. Docs mention that some meeting apps are "not supported yet" for capturing the other side, and that Teams inside Citrix/AVD does not deliver remote audio. Antivirus (Kaspersky) can block audio device access.
- **Mic picker:** "Auto" follows the mic the meeting app is using, or the system default; a specific device can be pinned and is remembered. Docs recommend switching off Bluetooth headset mics because they drop output to low-quality hands-free mode.
- **Linux:** no Linux app exists (confirmed by absence from all official docs and requirements pages).
- **Ad-hoc call detection:** Granola watches for any app using the microphone and offers a "take notes" prompt naming the app (huddle / call / meeting wording varies by app). Ad-hoc prompts are styled differently from calendar prompts (dashed vs solid accent bar). A call starting within 15 minutes after a scheduled event is assumed to be that event.
- **Audio retention:** audio is streamed to a third-party transcription provider and **not stored**. There is no way to replay or re-transcribe a meeting later. Pre-recorded audio files cannot be imported.

## 3. Transcription

- **Live**, streamed while the meeting runs, via an unnamed cloud provider (desktop). Mobile caches audio temporarily and deletes it after transcription.
- **Speaker labels (desktop default):** mic = "Me" (green bubbles, right side), system audio = "Them" (grey bubbles, left side). This is pure channel separation, not diarization.
- **Named speaker tags:** on Zoom, Teams (desktop apps) and Google Meet (Chrome via accessibility APIs or a Chrome extension), Granola reads participant names and the active-speaker indicator from the meeting app's UI and stamps names on transcript lines. Requires macOS Accessibility permission. Fails for shared room devices and overlapping speech. Not applied retroactively.
- **Mobile:** "Speaker A / Speaker B" diarization for in-person meetings.
- **Languages:** 31 on desktop (official list; changelog says 32 as of 2026-09), 17 on mobile. Settings: transcription = English or Multi-language; summary language = English or Auto (majority language of transcript). Language can be switched mid-meeting (brief pause while a model loads). UI itself is English only.
- **Custom vocabulary ("jargon"):** up to 30 personal terms and 50 workspace terms, passed as recognition hints.
- **Transcript controls:** show/hide, stop/resume, copy all, copy or delete individual chunks, search within transcript. Transcript text itself cannot be edited.
- **Limits:** back-to-back calls can merge if the meeting app holds the mic; long recordings (~10h+) may fail on mobile.

## 4. Note enhancement and the "yours vs AI" distinction

- Enhanced notes are a separate view from the user's raw notes. A tab/toggle switches between "my notes" and "enhanced". Edits in the raw view are taken into account on the next regeneration.
- Inside the enhanced view, text that came from the user's own notes is rendered in solid black (dark) text; the docs call this "highlighted in black". The implication, consistent with reviews that say "whatever you typed is highlighted", is that AI-written text is rendered in a lighter grey. **The exact AI-text color is UNVERIFIED (inferred grey).**
- **Provenance:** a magnifying-glass control beside each enhanced line reveals the transcript excerpt or raw note it was derived from. Share-link viewers can hover points to see transcript context.
- **Editing:** enhanced notes are directly editable; edits affect only that note, not future generations.
- **Regenerate:** a template button at the top-right regenerates using a different template; a repeat/refresh control regenerates with the same template. A warning appears before regenerating over edits (changelog 2026-01-08, "smarter regeneration warnings").
- **Rewrite via chat:** "make next steps more detailed", "reorder chronologically", "translate", etc. Selected text can be passed to chat.
- **"Enhancing notes" in-progress state / animation:** **UNVERIFIED**. No public source describes it in detail.
- **Undo after enhance:** **UNVERIFIED** (no public mention).
- Models: a blend, primarily OpenAI and Anthropic.

## 5. Templates and recipes

**Templates** (shape of the enhanced note)
- Default is an "Auto" template that adapts to the meeting type.
- 29 built-in templates (changelog 2024-10-07). Names publicly mentioned include 1:1, stand-up, interview, sales call / pitch. **The full list of 29 names is UNVERIFIED.** The set shown to a user depends on the "user type" chosen at sign-up (changeable in profile).
- Custom templates: free-form prompt describing purpose and context, length and style, and the desired section outline. Private by default; can be shared with everyone on the same company email domain.
- Switch during a call with `/` in the editor, or after enhancement from the template dropdown on the enhanced tab (which regenerates).
- Setting a default template other than Auto is **not supported** (a stated feature request).
- Templates are desktop-only.

**Recipes** (saved chat prompts)
- Saved prompts run with `/` in the floating chat bar. Each has a name (its slash shortcut), a prompt, and a scope: *single meeting* or *multiple meetings* (home, folders, people/company views, multi-select).
- Sections: default/Discover (curated), My recipes, Workspace recipes. Shareable: private, workspace, anyone with link.
- No variables or placeholders.
- Model choice per recipe on paid plans only.

## 6. Chat

- Scope follows location: home = all notes; inside a meeting = that meeting (before, during or after it); inside a folder or space = that folder; person/company view = that entity's meetings; multi-select = chosen meetings.
- Live in-meeting use: "what did I miss?", explain a topic, recap actions.
- Rebuilt as "agentic" in 2026-04 with **inline citations**. Citation format is **UNVERIFIED**.
- Agentic actions (desktop, need connectors): draft and send Gmail email; draft and send Slack message; find a time and create a Google Calendar event. Each produces an editable draft that requires confirmation.
- Attach files for extra context (not on iOS).
- Chat history is private even on shared notes; threads can be deleted.
- Model: Auto on free; Auto or a chosen standard or "thinking" model on paid.
- Shortcut: Ctrl/Cmd+J per the AI-enhanced notes article; a 2025 press item mentions Cmd+T with voice input. **Current binding UNVERIFIED (conflicting sources).** Chat supports dictation input.
- History window: last 30 days on free; full history on paid.

## 7. Organization

- **Spaces:** "My notes" (private) and a team space; admins can create custom spaces for groups.
- **Folders:** name, icon, description (description also helps chat). One level of nesting. A note can be in multiple folders. Bulk select (hover checkbox, shift-click range, select all).
- **Auto-file recurring meetings:** when you file one instance of a recurring event, Granola offers to auto-add future instances to that folder.
- **Related meetings** are linked by recurring-event ID or exact title match.
- **Search:** fast, works offline, jumps straight to people and companies.
- **People and Companies:** built automatically from calendar attendee lists (only for notes created from calendar events; declined attendees excluded). Enriched in the background with photos, job titles and company info from external data sources. Users cannot add people manually; corrections go through support.
- **@mentions** in notes, with attendee cards (2025-12 / 2026-01).
- **Trash:** soft delete, restore, 30-day retention, then permanent deletion.
- **Tags:** not mentioned anywhere. **UNVERIFIED / likely absent.**
- **Calendar:** Google and Microsoft 365 (cloud only); Apple Calendar not supported. Primary calendar by default, others toggleable. Declined events and non-meeting blocks (OOO, focus time) are hidden. Past events are not imported.

## 8. Sharing and export

- **Share link** per note with access levels (private / company / anyone with link) and roles (viewer can read and chat; collaborator can edit and apply templates). Web viewers see summarized notes only, no transcript. Access-request flow for outsiders.
- **Copy notes** → markdown on the clipboard (summary only). Paste-to-Slack formatting fixed in 2026-01; Ctrl/Cmd+Shift+V pastes without formatting. Plain text or HTML copy as explicit options: **UNVERIFIED**.
- **Copy transcript** from the transcript panel.
- **Email:** opens a draft in Gmail or Outlook with the note.
- **Export:** account-wide CSV (title, summary, transcript, metadata), emailed within hours, once per 24 hours. No per-note file export documented.
- **"Shared with me"** view.

## 9. Integrations

| Integration | Behavior (public docs) |
|---|---|
| Slack | Post a link plus summary to a channel manually, or auto-post every new note added to a folder. Channels only. Requires a company Google/Microsoft account. |
| Notion | Manual one-at-a-time "save to Notion" into a Granola-created database (no choice of target). Auto only via Zapier. |
| Zapier | Triggers: note added to folder; note manually shared to Zapier. Payload: title, creator, attendees, calendar event, private notes, markdown summary, transcript, share link. Paid only. |
| HubSpot / Attio / Affinity | Match notes to CRM people/company/deal records automatically. Paid. Salesforce: Enterprise. |
| Gmail | Follow-up email drafts and chat send actions. Paid for follow-ups. Google sign-in only. |
| Google Calendar / Outlook | Event sync; chat can create Google events. |
| Granola MCP | Remote MCP (OAuth) exposing query, list meetings and folders, get notes, transcripts, attachments. Free = last 30 days and no transcripts. |
| Granola API + webhooks | Paid. List/get notes, transcripts, folders, spaces; webhook endpoints; audit and legal-hold APIs (Enterprise). |
| Chrome extension | Google Meet speaker tags. |
| URL scheme | A custom URL scheme to create a new note. |

## 10. Settings, onboarding, permissions

- **Onboarding:** download → install → sign in with Google or Microsoft only (browser OAuth) → grant calendar read (attendees, title, time) → macOS only: grant mic and system-audio permission → 2-minute demo meeting with a pre-made demo note → home with upcoming meetings.
- **Settings areas seen in docs:** Profile (job info and context that steers summaries, CSV export), Preferences (Language and jargon, Data & sharing: training opt-out and default link-sharing level, Labs/beta channel, Features such as follow-up emails, reposition window for meetings), Notifications (global and per app), Calendar (calendar toggles), Connectors/Integrations, Templates, Workspace/team (members, spaces, workspace jargon, data security toggles), Troubleshooting (network check), Subscription.
- **Transparency features:** optional auto-posted chat message in Meet/Zoom/Teams/Slack huddles and a "watermark" telling participants Granola is in use; admin-enforced on Enterprise.
- **Account:** workspaces, account switcher, transfer notes between workspaces or accounts.

## 11. Pricing and gating (granola.ai/pricing, 2026-10-18)

| Tier | Price |
|---|---|
| Basic | $0 per user per month |
| Business | $14 per user per month |
| Enterprise | $35 per user per month |

Key gates: **30-day history** on Basic (notes older than 30 days hidden in app and chat); pre-meeting briefs, follow-up emails, workspace dictionary, Notion/Zapier/HubSpot/Attio/Affinity, API and model selection are Business+; Salesforce, SSO/SCIM, HIPAA, audit API, usage analytics and admin-wide policies are Enterprise. Free tier includes AI notes, bot-free capture, all apps, speaker tags, custom templates, multi-language, recipes, shared folders, Slack and MCP (limited).

## 12. Platforms

- Desktop: macOS 14+ and Windows. Linux: none.
- iOS (2025-04, in-person meetings and phone calls), Android (2026-07, no phone-call capture), Apple Watch (2026-07; watchOS 11+; records offline, syncs via iPhone, pipeline states Saved → Syncing → Synced → Uploading → Generating → Complete).
- Web: view and edit only, no capture.
- Offline: read and edit notes offline (2024-12). Capture needs the network (cloud transcription).

## 13. UI observations

From docs, changelog and reviews (no screenshots were copied):

- **Layout:** left sidebar (spaces and folders; people and companies icons and trash at the bottom; workspace/account menu bottom-left) → main pane with home (coming-up strip on top, notes list below) or note view. Note view: editor center, transcript panel toggled from the meeting controls, a floating chat bar docked at the bottom, an images stack above chat, template control top-right.
- **Feel:** minimal, calm, document-like, plenty of whitespace, notes read like a doc not a dashboard. Reviewers consistently praise the restraint. Specific fonts and spacing values are **UNVERIFIED** (and must not be copied anyway).
- **Focus mode (2026-04):** window chrome fades out while you type.
- **Dark mode** since 2024-11.
- **Transcript bubbles:** chat-like; Me right (green), Them left (grey).
- **Prompts:** calendar notification with solid accent bar vs ad-hoc with dashed bar.
- **Keyboard shortcuts documented:** Ctrl/Cmd+B, I, U; markdown triggers (`#`, `-`/`*`, `[]`); `/` for templates in the editor and recipes in chat; Ctrl/Cmd+J (or Cmd+T) for chat; Ctrl/Cmd+Shift+V plain paste. **A full shortcut list is UNVERIFIED.**
- **Empty states, enhancing animation, loading skeletons:** **UNVERIFIED.**

---

## 14. Feature matrix

Legend for **Our plan**: milestone that ships it (M1–M12), or **Skip** / **Later** with a reason. "Free" means Granola Basic; "Paid" means Business and/or Enterprise. Every feature we ship is free, with no caps.

| Feature | Free tier | Paid tier | Our plan |
|---|---|---|---|
| Bot-free capture: mic + system audio | Yes | Yes | M2 (Windows WASAPI loopback first, then macOS, then Linux) |
| Mic device picker with "Auto" | Yes | Yes | M2 |
| Level meter / live recording indicator | Yes (indicator) | Yes | M2 (meter + tray/overlay indicator) |
| Crash-safe capture | Not stated (audio never stored) | Not stated | M2: chunked WAV on disk with recovery on next launch |
| Keep audio locally / replay | No (audio never stored) | No | M2/M3: kept locally by default, optional auto-delete after N days |
| Import pre-recorded audio file | No | No | M3 (also powers the integration test) |
| Live streaming transcription | Yes (cloud) | Yes | M3: local whisper.cpp with VAD-chunked streaming; optional BYOK Deepgram / OpenAI / Groq |
| Me vs Them labels | Yes | Yes | M3 (channel-based) |
| Named speaker tags from Zoom/Meet/Teams UI | Yes (desktop) | Yes | Later: accessibility-API scraping of meeting apps is fragile and OS-specific; ship "Them" plus manual rename and optional AI name inference first |
| Diarization of "Them" (Speaker A/B) | Mobile only | Mobile only | UNVERIFIED feasibility locally; evaluate after M3 (possible with a local embedding model) |
| Multi-language transcription, mid-meeting switch | Yes (31) | Yes | M3: whisper's ~99 languages, per-meeting language or auto-detect |
| Summary language (English / auto) | Yes | Yes | M5 |
| Custom vocabulary (jargon) | 30 personal | +50 workspace | M3: unlimited list passed as whisper initial prompt / cloud keyword hints |
| Delete transcript chunks, copy, search | Yes | Yes | M3 |
| Edit transcript text | No | No | M3 (we allow it) |
| Rich-text notepad with markdown shortcuts | Yes | Yes | M4 |
| Images in notes (drop/paste) | Yes | Yes | M4 |
| @mentions with attendee cards | Yes | Yes | M8 |
| Auto-generate enhanced notes at meeting end | Yes | Yes | M5 |
| User text vs AI text visually distinct | Yes | Yes | M5 (own visual treatment; user text marked at the block/inline-span level) |
| Provenance ("show source" per line) | Yes | Yes | M5/M7: each AI block links to transcript timestamps |
| Regenerate, switch template, regeneration warning | Yes | Yes | M5/M6 |
| Undo enhance | UNVERIFIED | UNVERIFIED | M5: version history per note, one-click revert |
| Rewrite note via chat / selection | Yes | Yes | M7 |
| "Enhancing notes" streaming state | UNVERIFIED | UNVERIFIED | M5 (our own design) |
| Built-in templates | Yes (29, names UNVERIFIED) | Yes | M6: 12+ of our own written from scratch |
| Custom templates | Yes | Yes | M6 |
| Set default template | No | No | M6 (we allow it, plus per-folder/per-recurring-event defaults) |
| Share templates | Same email domain | Same | M10: import/export template files (JSON/markdown) |
| Recipes (saved chat prompts, `/`) | Yes | Yes + model choice | M7, with model choice for everyone |
| Chat: single meeting (also live) | Yes | Yes | M7 |
| Chat: folder / selection / person / company / all | Yes (30-day history) | Yes (unlimited) | M7 (unlimited; FTS5 retrieval plus optional local embeddings) |
| Inline citations to transcript | Yes (format UNVERIFIED) | Yes | M7: clickable timestamp citations |
| Chat model selection | Auto only | Yes | M7, for everyone (any configured provider) |
| Chat file attachments | Yes | Yes | M7 (text/markdown/PDF text) |
| Agentic send email / Slack / create calendar event | Yes (connectors) | Yes | Partial: Slack via webhook in M11; email as mailto/copy draft in M10; calendar event creation **Skip** (write scope risk; read-only calendar) |
| Pre-meeting briefs | No | Yes | M9: local brief from past notes with the same attendees/company. No web/LinkedIn lookup (privacy) |
| Follow-up email drafts | No | Yes | M10: generated draft, open in default mail client or copy. No Gmail send API |
| Folders, nesting, multi-folder notes | Yes | Yes | M8 (unlimited nesting) |
| Auto-file recurring meetings | Yes | Yes | M8/M9 |
| Spaces (private / team / custom) | Yes | Yes | Skip: single-user local-first app; folders cover it. Revisit with sync |
| Full-text search | Yes | Yes | M8 (SQLite FTS5 across notes, enhanced notes, transcripts) |
| People and companies from attendees | Yes | Yes | M8 (from attendees and email domains; editable, mergeable, manual add) |
| Web enrichment of people (photo, title) | Yes | Yes | Skip: requires sending contacts to third parties; conflicts with local-first |
| Tags | UNVERIFIED (likely absent) | UNVERIFIED | M8 |
| Trash with 30-day restore | Yes | Yes | M8 |
| Meeting history limit | 30 days | Unlimited | Unlimited, always |
| Google Calendar sync | Yes | Yes | M9 (OAuth with the user's own client credentials) |
| Outlook / Microsoft 365 calendar sync | Yes | Yes | M9 (same approach) |
| Apple Calendar | No | No | Later: ICS URL subscription in M9 covers many cases |
| Pre-meeting notification, open call link | Yes | Yes | M9 |
| Ad-hoc call detection (mic in use) | Yes | Yes | M9 on Windows (mic-in-use registry/session APIs); macOS/Linux later |
| Window repositioning on meeting start | Yes | Yes | M12 (optional) |
| Share link | Yes | Yes | M10: optional self-hostable static share server; off by default |
| Copy as markdown | Yes | Yes | M10 |
| Copy as plain text / HTML | UNVERIFIED | UNVERIFIED | M10 |
| Export to file | CSV account export only | Same | M10: per-note .md/.txt/.html, bulk zip, JSON backup |
| Slack | Yes | Yes | M11: incoming webhook, manual + per-folder auto-post |
| Notion | No | Yes | M11: user's own integration token, target database of their choice |
| Zapier | No | Yes | M11: Zapier-compatible webhook (catch hook) with same kind of payload |
| Generic webhook | No (API webhooks paid) | Yes | M11 (with HMAC signing) |
| HubSpot / Attio / Affinity / Salesforce | No | Yes / Enterprise | Later: webhook + Zapier cover it; native CRM adapters after v1 |
| Markdown folder sync (Obsidian) | No | No | M11 |
| Public API | No | Yes | Later: local MCP server exposing notes to the user's own AI tools is a strong follow-up |
| MCP access | Limited | Yes | Later (see above) |
| Screen snapshots of shared screens | macOS only | Same | Later: needs per-OS window capture plus a consent UX |
| Transparency message / watermark | Per user | Admin enforced | M12: "copy consent message" helper; no meeting-app automation |
| Dark mode | Yes | Yes | M12 (light/dark/system) |
| Focus mode | Yes | Yes | M12 |
| Keyboard shortcuts / command palette | Partial (list UNVERIFIED) | Same | M12 (full list plus command palette) |
| Offline use | Read/edit only | Same | Full: capture, transcribe (local whisper) and enhance (Ollama) all offline |
| Local LLM | No | No | M5 via Ollama |
| BYOK cloud AI | No | No | M5 (Anthropic, OpenAI, Gemini, OpenRouter) |
| Model-training opt-out | Per user | Workspace | N/A: we never send data anywhere the user did not configure |
| Telemetry | Yes (not stated in detail) | Yes | None by default |
| Onboarding with permission flow | Yes | Yes | M1 |
| Profile context steering summaries | Yes | Yes | M1 (settings) / M5 (prompt) |
| Beta channel | Yes | Yes | Phase 4 (GitHub pre-releases) |
| i18n of UI | English only | English only | M12 scaffolding |
| Windows / macOS desktop | Yes | Yes | Windows first (M1–M12), macOS capture adapter, Linux capture adapter |
| Linux desktop | No | No | Yes (PipeWire/PulseAudio monitor) |
| iOS / Android / Apple Watch | Yes | Yes | Skip for v1: Tauri mobile is viable later, but capture and background rules differ per OS |
| Web app | View/edit | Same | Skip (local-first desktop) |
| SSO, SCIM, audit, legal hold, HIPAA, SOC 2, admin analytics | No | Enterprise | Skip: enterprise multi-tenant features with no meaning for a local single-user app |
| Workspaces, teammate invites, account transfer | Yes | Yes | Skip for v1 (no accounts); JSON backup/import in M10 covers moving data |
| Year-in-review ("Crunched") | Yes | Yes | Skip (novelty) |

## Sources

Official (public pages only):
- Pricing: https://www.granola.ai/pricing
- Changelog: https://www.granola.ai/updates
- Help-center index: https://docs.granola.ai/llms.txt
- Setup guide, Granola 101, AI-enhanced notes, writing your own notes, templates, recipes, chatting with meetings, workflows in chat, transcription, speaker tags, multi-language, customizing transcription, notifications, calendar sync, sharing, spaces and folders, integrations overview, Slack, Notion, Zapier, MCP, exporting, people and companies, profile and preferences, pre-meeting briefs, follow-up emails, screen snapshots, deleting notes, transcription issues, mobile transcription, Apple Watch, common feature requests: all under https://docs.granola.ai/help-center/
- Teams and WebEx bot-free blog posts: https://granola.ai/blog/granola-microsoft-teams-bot-free-notes, https://www.granola.ai/blog/granola-cisco-webex-bot-free-meeting-notes

Third-party (lower confidence, used only where noted):
- HappyScribe review (typed-text highlighting): https://www.happyscribe.com/de/blog/granola-ai-test
- Focus mode: https://createwith.com/tool/granola/updates/granola-adds-focus-mode-to-reduce-distractions-while-taking-meeting-notes
- Chat shortcut report: https://alternativeto.net/news/2025/10/granola-adds-chat-feature-with-voice-activation-and-cmd-t-shortcut
- Linux absence: https://meetily.ai/granola-for-linux (competitor page)
- Pricing aggregators (cross-check only): https://www.meetjamie.ai/en/blog/granola-ai-pricing, https://toolradar.com/tools/granola/pricing
