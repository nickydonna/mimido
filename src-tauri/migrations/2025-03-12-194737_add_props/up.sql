-- Your SQL goes here

ALTER TABLE `events` DROP COLUMN `etag`;
ALTER TABLE `events` DROP COLUMN `url`;
ALTER TABLE `events` ADD COLUMN `summary` TEXT NOT NULL;
ALTER TABLE `events` ADD COLUMN `href` TEXT NOT NULL;
ALTER TABLE `events` ADD COLUMN `ends_at` TEXT NOT NULL;
ALTER TABLE `events` ADD COLUMN `recur` TEXT;
ALTER TABLE `events` ADD COLUMN `description` TEXT;
ALTER TABLE `events` ADD COLUMN `starts_at` TEXT NOT NULL;




