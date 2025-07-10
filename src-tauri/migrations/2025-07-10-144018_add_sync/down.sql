-- This file should undo anything in `up.sql`
ALTER TABLE `calendar` DROP COLUMN `is_default`;
ALTER TABLE `calendars` ADD COLUMN `default_value` BOOL NOT NULL DEFAULT FALSE;
