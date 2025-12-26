-- This file should undo anything in `up.sql`
ALTER TABLE `vtodos` DROP COLUMN `synced_at`;
ALTER TABLE `vevents` DROP COLUMN `synced_at`;
ALTER TABLE `vtodos` DROP COLUMN `last_modified`;
ALTER TABLE `vevents` DROP COLUMN `last_modified`;

ALTER TABLE `vtodos` ADD COLUMN `synced_at` BIGINT NOT NULL DEFAULT 0;
ALTER TABLE `vevents` ADD COLUMN `synced_at` BIGINT NOT NULL DEFAULT 0;
ALTER TABLE `vtodos` ADD COLUMN `last_modified` BIGINT NOT NULL DEFAULT 0;
ALTER TABLE `vevents` ADD COLUMN `last_modified` BIGINT NOT NULL DEFAULT 0;
