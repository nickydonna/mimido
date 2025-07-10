-- Your SQL goes here
ALTER TABLE `calendars` ADD COLUMN `is_default` BOOL NOT NULL DEFAULT FALSE;
ALTER TABLE `calendars` ADD COLUMN `sync_token` TEXT;
