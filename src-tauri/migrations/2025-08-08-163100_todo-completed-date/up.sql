-- Your SQL goes here
-- make completed a date
ALTER TABLE `vtodos` DROP COLUMN `completed`;
ALTER TABLE `vtodos` ADD COLUMN `completed` TEXT;
