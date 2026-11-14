-- Nurbit server database, version 1. Append-only: nothing is overwritten.

-- First version of every record id received from any device.
CREATE TABLE synced_records (
    id           TEXT PRIMARY KEY,
    kind         TEXT NOT NULL CHECK (kind IN ('crew_member', 'check_in')),
    content_hash TEXT NOT NULL,
    payload      TEXT NOT NULL,      -- JSON
    device_id    TEXT NOT NULL,
    received_at  INTEGER NOT NULL
);
CREATE INDEX idx_synced_kind ON synced_records (kind);

-- Later versions of an id whose content differs: kept for specialist review.
CREATE TABLE sync_conflicts (
    seq          INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id    TEXT NOT NULL REFERENCES synced_records (id),
    content_hash TEXT NOT NULL,
    payload      TEXT NOT NULL,
    device_id    TEXT NOT NULL,
    received_at  INTEGER NOT NULL,
    UNIQUE (record_id, content_hash)
);

CREATE TRIGGER synced_append_only_update BEFORE UPDATE ON synced_records BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE TRIGGER synced_append_only_delete BEFORE DELETE ON synced_records BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE TRIGGER conflicts_append_only_update BEFORE UPDATE ON sync_conflicts BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE TRIGGER conflicts_append_only_delete BEFORE DELETE ON sync_conflicts BEGIN SELECT RAISE(ABORT, 'append-only'); END;
