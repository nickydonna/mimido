-- Your SQL goes here
ALTER TABLE `events` ADD COLUMN `has_rrule` INTEGER NOT NULL DEFAULT 0;
ALTER TABLE `events` DROP COLUMN `recur`;
