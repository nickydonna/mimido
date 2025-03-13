-- This file should undo anything in `up.sql`

ALTER TABLE `events` DROP COLUMN `summary`;
ALTER TABLE `events` DROP COLUMN `href`;
ALTER TABLE `events` DROP COLUMN `ends_at`;
ALTER TABLE `events` DROP COLUMN `recur`;
ALTER TABLE `events` DROP COLUMN `description`;
ALTER TABLE `events` DROP COLUMN `starts_at`;
ALTER TABLE `events` ADD COLUMN `etag` TEXT NOT NULL;
ALTER TABLE `events` ADD COLUMN `url` TEXT NOT NULL;




