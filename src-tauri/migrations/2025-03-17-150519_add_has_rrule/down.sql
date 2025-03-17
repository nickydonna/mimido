-- This file should undo anything in `up.sql`
ALTER TABLE `events` DROP COLUMN `has_rrule`;
ALTER TABLE `events` ADD COLUMN `recur` TEXT;
