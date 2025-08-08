-- Your SQL goes here
ALTER TABLE `vtodos` ADD COLUMN `has_rrule`  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE `vtodos` ADD COLUMN `rrule_str` TEXT;

