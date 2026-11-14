-- Version 4: log of assistant questions and which sources were shown.
-- Automated output, kept apart from specialist decisions. Append-only.

CREATE TABLE assistant_log (
    id              TEXT PRIMARY KEY,
    crew_member_id  TEXT NOT NULL REFERENCES crew_members (id),
    created_at      INTEGER NOT NULL,
    provider        TEXT NOT NULL,
    question        TEXT NOT NULL,
    body_area       TEXT,
    severity        INTEGER,
    matched_entries TEXT NOT NULL,   -- comma-separated knowledge-base entry ids
    urgent          INTEGER NOT NULL CHECK (urgent IN (0, 1))
);

CREATE TRIGGER assistant_log_append_only_update BEFORE UPDATE ON assistant_log BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER assistant_log_append_only_delete BEFORE DELETE ON assistant_log BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
