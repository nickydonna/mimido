-- Your SQL goes here
ALTER TABLE `events` ADD COLUMN `has_rrule` INTEGER NOT NULL;
ALTER TABLE `events` DROP COLUMN `recur`;
