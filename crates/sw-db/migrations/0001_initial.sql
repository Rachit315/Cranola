-- Scribewell schema v1.
-- Conventions: ids are UUIDv7 text; times are Unix milliseconds (INTEGER);
-- soft delete via deleted_at; STRICT tables for type safety.

CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value_json TEXT NOT NULL CHECK (json_valid(value_json)),
    updated_at INTEGER NOT NULL
) STRICT;

-- Organization ---------------------------------------------------------------

CREATE TABLE companies (
    id     TEXT PRIMARY KEY,
    name   TEXT NOT NULL,
    domain TEXT UNIQUE COLLATE NOCASE,
    notes  TEXT NOT NULL DEFAULT ''
) STRICT;

CREATE TABLE people (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    email       TEXT UNIQUE COLLATE NOCASE,
    company_id  TEXT REFERENCES companies(id) ON DELETE SET NULL,
    notes       TEXT NOT NULL DEFAULT '',
    merged_into TEXT REFERENCES people(id) ON DELETE SET NULL
) STRICT;

CREATE TABLE templates (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    icon        TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    is_builtin  INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0, 1)),
    spec_json   TEXT NOT NULL CHECK (json_valid(spec_json)),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
) STRICT;

CREATE TABLE folders (
    id                  TEXT PRIMARY KEY,
    parent_id           TEXT REFERENCES folders(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    icon                TEXT NOT NULL DEFAULT '',
    color               TEXT NOT NULL DEFAULT '',
    description         TEXT NOT NULL DEFAULT '',
    position            INTEGER NOT NULL DEFAULT 0,
    default_template_id TEXT REFERENCES templates(id) ON DELETE SET NULL,
    created_at          INTEGER NOT NULL,
    deleted_at          INTEGER
) STRICT;

CREATE TABLE tags (
    id    TEXT PRIMARY KEY,
    name  TEXT NOT NULL UNIQUE COLLATE NOCASE,
    color TEXT NOT NULL DEFAULT ''
) STRICT;

-- Calendar -------------------------------------------------------------------

CREATE TABLE calendar_accounts (
    id          TEXT PRIMARY KEY,
    provider    TEXT NOT NULL CHECK (provider IN ('google', 'microsoft', 'ics')),
    label       TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(config_json))
) STRICT;

CREATE TABLE calendars (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL REFERENCES calendar_accounts(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL DEFAULT '',
    enabled     INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    UNIQUE (account_id, external_id)
) STRICT;

CREATE TABLE calendar_events (
    id             TEXT PRIMARY KEY,
    calendar_id    TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    external_id    TEXT NOT NULL,
    series_id      TEXT,
    title          TEXT NOT NULL,
    start_at       INTEGER NOT NULL,
    end_at         INTEGER NOT NULL,
    all_day        INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    conference_url TEXT,
    location       TEXT,
    attendees_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(attendees_json)),
    my_response    TEXT,
    status         TEXT NOT NULL DEFAULT 'confirmed',
    etag           TEXT,
    UNIQUE (calendar_id, external_id)
) STRICT;
CREATE INDEX calendar_events_start ON calendar_events(start_at);

-- Meetings and capture -------------------------------------------------------

CREATE TABLE meetings (
    id                TEXT PRIMARY KEY,
    title             TEXT NOT NULL DEFAULT '',
    status            TEXT NOT NULL DEFAULT 'draft'
                      CHECK (status IN ('draft', 'recording', 'processing', 'ready', 'failed')),
    scheduled_start   INTEGER,
    scheduled_end     INTEGER,
    started_at        INTEGER,
    ended_at          INTEGER,
    calendar_event_id TEXT REFERENCES calendar_events(id) ON DELETE SET NULL,
    template_id       TEXT REFERENCES templates(id) ON DELETE SET NULL,
    language          TEXT,
    summary_language  TEXT,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    deleted_at        INTEGER
) STRICT;
CREATE INDEX meetings_created ON meetings(created_at DESC);

CREATE TABLE meeting_folders (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    folder_id  TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    PRIMARY KEY (meeting_id, folder_id)
) STRICT;

CREATE TABLE folder_rules (
    id          TEXT PRIMARY KEY,
    folder_id   TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('recurring_series')),
    match_value TEXT NOT NULL
) STRICT;

CREATE TABLE meeting_tags (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    tag_id     TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (meeting_id, tag_id)
) STRICT;

CREATE TABLE meeting_attendees (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    person_id  TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role       TEXT NOT NULL DEFAULT 'attendee' CHECK (role IN ('organizer', 'attendee')),
    response   TEXT,
    PRIMARY KEY (meeting_id, person_id)
) STRICT;

CREATE TABLE recordings (
    id              TEXT PRIMARY KEY,
    meeting_id      TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    started_at      INTEGER NOT NULL,
    ended_at        INTEGER,
    status          TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'finalized', 'recovered', 'failed')),
    mic_device      TEXT,
    loopback_device TEXT,
    sample_rate     INTEGER NOT NULL,
    dir             TEXT NOT NULL,
    error           TEXT
) STRICT;
CREATE INDEX recordings_status ON recordings(status);

CREATE TABLE audio_chunks (
    recording_id TEXT NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
    channel      TEXT NOT NULL CHECK (channel IN ('me', 'them')),
    seq          INTEGER NOT NULL,
    path         TEXT NOT NULL,
    start_ms     INTEGER NOT NULL,
    duration_ms  INTEGER NOT NULL,
    PRIMARY KEY (recording_id, channel, seq)
) STRICT;

CREATE TABLE audio_files (
    recording_id TEXT NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
    channel      TEXT NOT NULL CHECK (channel IN ('me', 'them')),
    path         TEXT NOT NULL,
    codec        TEXT NOT NULL,
    duration_ms  INTEGER NOT NULL,
    bytes        INTEGER NOT NULL,
    PRIMARY KEY (recording_id, channel)
) STRICT;

-- Transcript -----------------------------------------------------------------

CREATE TABLE transcript_segments (
    id           TEXT PRIMARY KEY,
    meeting_id   TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    recording_id TEXT REFERENCES recordings(id) ON DELETE SET NULL,
    channel      TEXT NOT NULL CHECK (channel IN ('me', 'them')),
    start_ms     INTEGER NOT NULL,
    end_ms       INTEGER NOT NULL,
    text         TEXT NOT NULL,
    avg_logprob  REAL,
    engine       TEXT NOT NULL,
    model        TEXT NOT NULL,
    language     TEXT,
    edited       INTEGER NOT NULL DEFAULT 0 CHECK (edited IN (0, 1)),
    deleted_at   INTEGER
) STRICT;
CREATE INDEX transcript_segments_meeting ON transcript_segments(meeting_id, start_ms);

CREATE TABLE meeting_speakers (
    meeting_id   TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    channel      TEXT NOT NULL CHECK (channel IN ('me', 'them')),
    display_name TEXT NOT NULL,
    person_id    TEXT REFERENCES people(id) ON DELETE SET NULL,
    PRIMARY KEY (meeting_id, channel)
) STRICT;

-- Notes ----------------------------------------------------------------------

CREATE TABLE documents (
    id                 TEXT PRIMARY KEY,
    meeting_id         TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    kind               TEXT NOT NULL CHECK (kind IN ('raw', 'enhanced')),
    current_version_id TEXT,
    updated_at         INTEGER NOT NULL,
    UNIQUE (meeting_id, kind)
) STRICT;

CREATE TABLE document_versions (
    id           TEXT PRIMARY KEY,
    document_id  TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    content_json TEXT NOT NULL CHECK (json_valid(content_json)),
    plain_text   TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    cause        TEXT NOT NULL CHECK (cause IN ('typing', 'enhance', 'regenerate',
                 'template_switch', 'chat_rewrite', 'restore')),
    template_id  TEXT REFERENCES templates(id) ON DELETE SET NULL,
    llm_run_id   TEXT
) STRICT;
CREATE INDEX document_versions_doc ON document_versions(document_id, created_at DESC);

CREATE TABLE note_blocks (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    block_id    TEXT NOT NULL,
    position    INTEGER NOT NULL,
    block_type  TEXT NOT NULL,
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mixed')),
    text        TEXT NOT NULL,
    PRIMARY KEY (document_id, block_id)
) STRICT;

CREATE TABLE block_citations (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    block_id    TEXT NOT NULL,
    segment_id  TEXT NOT NULL REFERENCES transcript_segments(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, block_id, segment_id)
) STRICT;

CREATE TABLE recipes (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE COLLATE NOCASE,
    prompt         TEXT NOT NULL,
    scope          TEXT NOT NULL CHECK (scope IN ('single', 'multi')),
    model_override TEXT,
    is_builtin     INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0, 1))
) STRICT;

-- Chat -----------------------------------------------------------------------

CREATE TABLE chats (
    id             TEXT PRIMARY KEY,
    scope_type     TEXT NOT NULL
                   CHECK (scope_type IN ('meeting', 'folder', 'person', 'company', 'selection', 'all')),
    scope_ref_json TEXT NOT NULL DEFAULT 'null' CHECK (json_valid(scope_ref_json)),
    title          TEXT NOT NULL DEFAULT '',
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
) STRICT;

CREATE TABLE chat_messages (
    id             TEXT PRIMARY KEY,
    chat_id        TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    role           TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
    content        TEXT NOT NULL,
    citations_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(citations_json)),
    llm_run_id     TEXT,
    created_at     INTEGER NOT NULL
) STRICT;
CREATE INDEX chat_messages_chat ON chat_messages(chat_id, created_at);

-- Integrations ---------------------------------------------------------------

CREATE TABLE integrations (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('slack', 'notion', 'webhook', 'zapier', 'markdown_sync')),
    name        TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(config_json)),
    enabled     INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1))
) STRICT;

CREATE TABLE integration_rules (
    id             TEXT PRIMARY KEY,
    integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    trigger        TEXT NOT NULL CHECK (trigger IN ('manual', 'enhanced', 'added_to_folder')),
    folder_id      TEXT REFERENCES folders(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE deliveries (
    id              TEXT PRIMARY KEY,
    integration_id  TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    meeting_id      TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    trigger         TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN ('pending', 'delivered', 'failed')),
    attempts        INTEGER NOT NULL DEFAULT 0,
    next_attempt_at INTEGER,
    last_error      TEXT,
    created_at      INTEGER NOT NULL
) STRICT;

-- Diagnostics and search -----------------------------------------------------

CREATE TABLE llm_runs (
    id            TEXT PRIMARY KEY,
    task          TEXT NOT NULL,
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    input_tokens  INTEGER,
    output_tokens INTEGER,
    started_at    INTEGER NOT NULL,
    duration_ms   INTEGER,
    status        TEXT NOT NULL CHECK (status IN ('running', 'ok', 'error', 'cancelled')),
    error_code    TEXT
) STRICT;

CREATE TABLE embeddings (
    owner_type TEXT NOT NULL,
    owner_id   TEXT NOT NULL,
    model      TEXT NOT NULL,
    dim        INTEGER NOT NULL,
    vector     BLOB NOT NULL,
    PRIMARY KEY (owner_type, owner_id, model)
) STRICT;

CREATE VIRTUAL TABLE search_index USING fts5(
    meeting_id UNINDEXED,
    kind UNINDEXED,
    ref_id UNINDEXED,
    title,
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);
