-- Your SQL goes here
ALTER TABLE `vtodos` DROP COLUMN `synced_at`;
ALTER TABLE `vevents` DROP COLUMN `synced_at`;
ALTER TABLE `vtodos` DROP COLUMN `last_modified`;
ALTER TABLE `vevents` DROP COLUMN `last_modified`;

ALTER TABLE `vtodos` ADD COLUMN `synced_at` TEXT;
ALTER TABLE `vevents` ADD COLUMN `synced_at` TEXT;
ALTER TABLE `vtodos` ADD COLUMN `last_modified` TEXT;
ALTER TABLE `vevents` ADD COLUMN `last_modified` TEXT;
