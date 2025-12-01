-- Your SQL goes here
-- Make href, ical_data, and etag nullable and add synced_at field

-- Create a new table with the updated schema
CREATE TABLE vevents_new (
    id INTEGER PRIMARY KEY NOT NULL,
    calendar_id INTEGER NOT NULL,
    uid TEXT NOT NULL,
    href TEXT,
    ical_data TEXT,
    summary TEXT NOT NULL,
    description TEXT,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    has_rrule BOOL NOT NULL,
    rrule_str TEXT,
    tag TEXT,
    status TEXT NOT NULL,
    event_type TEXT NOT NULL,
    original_text TEXT,
    load INTEGER NOT NULL,
    urgency INTEGER NOT NULL,
    importance INTEGER NOT NULL,
    postponed INTEGER NOT NULL,
    last_modified BIGINT NOT NULL,
    etag TEXT,
    synced_at BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (calendar_id) REFERENCES calendars(id)
);

-- Copy data from old table to new table
INSERT INTO vevents_new (
    id, calendar_id, uid, href, ical_data, summary, description,
    starts_at, ends_at, has_rrule, rrule_str, tag, status, event_type,
    original_text, load, urgency, importance, postponed, last_modified, etag, synced_at
)
SELECT
    id, calendar_id, uid, href, ical_data, summary, description,
    starts_at, ends_at, has_rrule, rrule_str, tag, status, event_type,
    original_text, load, urgency, importance, postponed, last_modified, etag, 0
FROM vevents;

-- Drop old table
DROP TABLE vevents;

-- Rename new table to original name
ALTER TABLE vevents_new RENAME TO vevents;
