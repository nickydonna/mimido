-- This file should undo anything in `up.sql`
ALTER TABLE `vtodos` DROP COLUMN `completed`;
ALTER TABLE `vtodos` ADD COLUMN `completed` BOOL NOT NULL DEFAULT FALSE;
;
