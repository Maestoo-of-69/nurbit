-- Version 3: specialist records (human judgment), stored apart from machine output.
-- All append-only: corrections are new rows, never edits.

CREATE TABLE specialist_notes (
    id             TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members (id),
    author_id      TEXT NOT NULL REFERENCES crew_members (id),
    created_at     INTEGER NOT NULL,
    text           TEXT NOT NULL
);

CREATE TABLE annotations (
    id              TEXT PRIMARY KEY,
    checkin_id      TEXT NOT NULL REFERENCES checkins (id),
    question_id     TEXT NOT NULL,
    author_id       TEXT NOT NULL REFERENCES crew_members (id),
    created_at      INTEGER NOT NULL,
    corrected_value TEXT NOT NULL,
    reason          TEXT NOT NULL
);

CREATE TABLE baseline_reviews (
    id             TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members (id),
    indicator      TEXT NOT NULL,
    decision       TEXT NOT NULL CHECK (decision IN ('approve_new_baseline', 'keep_current_baseline')),
    decided_by     TEXT NOT NULL REFERENCES crew_members (id),
    decided_at     INTEGER NOT NULL,
    note           TEXT NOT NULL
);

CREATE TABLE decisions (
    id             TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members (id),
    author_id      TEXT NOT NULL REFERENCES crew_members (id),
    created_at     INTEGER NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('follow_up_exam', 'clear_for_duty', 'duty_restriction', 'other')),
    text           TEXT NOT NULL
);

CREATE TABLE decision_reviews (
    id          TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL REFERENCES decisions (id),
    reviewer_id TEXT NOT NULL REFERENCES crew_members (id),
    created_at  INTEGER NOT NULL,
    verdict     TEXT NOT NULL CHECK (verdict IN ('confirm', 'reject')),
    comment     TEXT NOT NULL
);

CREATE TRIGGER notes_append_only_update BEFORE UPDATE ON specialist_notes BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER notes_append_only_delete BEFORE DELETE ON specialist_notes BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER annotations_append_only_update BEFORE UPDATE ON annotations BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER annotations_append_only_delete BEFORE DELETE ON annotations BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER baseline_reviews_append_only_update BEFORE UPDATE ON baseline_reviews BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER baseline_reviews_append_only_delete BEFORE DELETE ON baseline_reviews BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER decisions_append_only_update BEFORE UPDATE ON decisions BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER decisions_append_only_delete BEFORE DELETE ON decisions BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER decision_reviews_append_only_update BEFORE UPDATE ON decision_reviews BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
CREATE TRIGGER decision_reviews_append_only_delete BEFORE DELETE ON decision_reviews BEGIN SELECT RAISE(ABORT, 'records are append-only'); END;
