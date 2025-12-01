-- This file should undo anything in `up.sql`
-- Revert href, ical_data, and etag to NOT NULL and remove synced_at field

-- Create a new table with the old schema
CREATE TABLE vtodos_old (
    id INTEGER PRIMARY KEY NOT NULL,
    calendar_id INTEGER NOT NULL,
    uid TEXT NOT NULL,
    href TEXT NOT NULL,
    ical_data TEXT NOT NULL,
    summary TEXT NOT NULL,
    description TEXT,
    starts_at TEXT,
    ends_at TEXT,
    has_rrule INTEGER NOT NULL,
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
    etag TEXT NOT NULL,
    completed TEXT,
    FOREIGN KEY (calendar_id) REFERENCES calendars(id)
);

-- Copy data from current table to old table (only rows with non-null values)
INSERT INTO vtodos_old (
    id, calendar_id, uid, href, ical_data, summary, description,
    starts_at, ends_at, has_rrule, rrule_str, tag, status, event_type,
    original_text, load, urgency, importance, postponed, last_modified, etag, completed
)
SELECT
    id, calendar_id, uid,
    COALESCE(href, ''),
    COALESCE(ical_data, ''),
    summary, description,
    starts_at, ends_at, has_rrule, rrule_str, tag, status, event_type,
    original_text, load, urgency, importance, postponed, last_modified,
    COALESCE(etag, ''),
    completed
FROM vtodos;

-- Drop current table
DROP TABLE vtodos;

-- Rename old table to original name
ALTER TABLE vtodos_old RENAME TO vtodos;
