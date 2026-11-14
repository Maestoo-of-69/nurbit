-- Nurbit local database, version 1.
-- Times are integers: milliseconds since 1970-01-01 UTC.
-- Medical records are append-only: triggers below reject UPDATE and DELETE.

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE crew_members (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    role         TEXT NOT NULL CHECK (role IN ('astronaut', 'specialist')),
    position     TEXT NOT NULL,
    is_synthetic INTEGER NOT NULL CHECK (is_synthetic IN (0, 1)),
    created_at   INTEGER NOT NULL
);

CREATE TABLE checkins (
    id             TEXT PRIMARY KEY,               -- UUID v7 made on the device
    crew_member_id TEXT NOT NULL REFERENCES crew_members (id),
    started_at     INTEGER NOT NULL,
    completed_at   INTEGER NOT NULL CHECK (completed_at >= started_at),
    device_id      TEXT NOT NULL,
    origin         TEXT NOT NULL,                  -- self_report, simulated, ...
    recorded_at    INTEGER NOT NULL                -- when this database stored it
);
CREATE INDEX idx_checkins_member_time ON checkins (crew_member_id, completed_at DESC);

CREATE TABLE checkin_answers (
    checkin_id        TEXT NOT NULL REFERENCES checkins (id),
    question_id       TEXT NOT NULL,
    value_kind        TEXT NOT NULL CHECK (value_kind IN ('scale', 'yes_no', 'number', 'choice')),
    value_num         REAL,                        -- scale, yes_no (0/1), number
    value_text        TEXT,                        -- choice option id
    triggered_by_rule TEXT,                        -- NULL for mandatory questions
    PRIMARY KEY (checkin_id, question_id),
    CHECK ((value_kind = 'choice') = (value_text IS NOT NULL)),
    CHECK ((value_kind = 'choice') = (value_num IS NULL))
);

-- Records waiting to be sent to the medical server (Stage 10).
CREATE TABLE sync_outbox (
    record_id   TEXT PRIMARY KEY,
    record_type TEXT NOT NULL,
    queued_at   INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'conflict')),
    attempts    INTEGER NOT NULL DEFAULT 0,
    last_error  TEXT
);

CREATE TABLE audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    at          INTEGER NOT NULL,
    actor_id    TEXT,
    action      TEXT NOT NULL,
    record_type TEXT NOT NULL,
    record_id   TEXT NOT NULL
);

CREATE TRIGGER checkins_append_only_update BEFORE UPDATE ON checkins
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
CREATE TRIGGER checkins_append_only_delete BEFORE DELETE ON checkins
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
CREATE TRIGGER answers_append_only_update BEFORE UPDATE ON checkin_answers
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
CREATE TRIGGER answers_append_only_delete BEFORE DELETE ON checkin_answers
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
CREATE TRIGGER audit_append_only_update BEFORE UPDATE ON audit_log
BEGIN SELECT RAISE(ABORT, 'audit log is append-only'); END;
CREATE TRIGGER audit_append_only_delete BEFORE DELETE ON audit_log
BEGIN SELECT RAISE(ABORT, 'audit log is append-only'); END;
