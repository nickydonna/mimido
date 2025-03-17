-- This file should undo anything in `up.sql`
ALTER TABLE `events` DROP COLUMN `event_type`;
ALTER TABLE `events` DROP COLUMN `tag`;
ALTER TABLE `events` DROP COLUMN `status`;
ALTER TABLE `events` DROP COLUMN `original_text`;
ALTER TABLE `events` DROP COLUMN `importance`;
ALTER TABLE `events` DROP COLUMN `load`;
ALTER TABLE `events` DROP COLUMN `urgency`;
ALTER TABLE `events` DROP COLUMN `postponed`;

