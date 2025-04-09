-- This file should undo anything in `up.sql`
ALTER TABLE `todos` DROP COLUMN `event_type`;
ALTER TABLE `todos` DROP COLUMN `tag`;
ALTER TABLE `todos` DROP COLUMN `status`;
ALTER TABLE `todos` DROP COLUMN `original_text`;
ALTER TABLE `todos` DROP COLUMN `importance`;
ALTER TABLE `todos` DROP COLUMN `load`;
ALTER TABLE `todos` DROP COLUMN `urgency`;
ALTER TABLE `todos` DROP COLUMN `postponed`;
ALTER TABLE `todos` ADD COLUMN `etag` TEXT NOT NULL;
ALTER TABLE `todos` ADD COLUMN `url` TEXT NOT NULL;


