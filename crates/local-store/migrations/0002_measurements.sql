-- Version 2: numeric measurements from devices, exams or manual entry.
-- Append-only like check-ins.

CREATE TABLE measurements (
    id             TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members (id),
    indicator      TEXT NOT NULL,      -- e.g. exercise.resistive_minutes
    value          REAL NOT NULL,
    unit           TEXT NOT NULL,
    measured_at    INTEGER NOT NULL,
    origin         TEXT NOT NULL,      -- device, manual_entry, simulated, ...
    recorded_at    INTEGER NOT NULL
);
CREATE INDEX idx_measurements_member ON measurements (crew_member_id, indicator, measured_at DESC);

CREATE TRIGGER measurements_append_only_update BEFORE UPDATE ON measurements
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
CREATE TRIGGER measurements_append_only_delete BEFORE DELETE ON measurements
BEGIN SELECT RAISE(ABORT, 'medical records are append-only'); END;
