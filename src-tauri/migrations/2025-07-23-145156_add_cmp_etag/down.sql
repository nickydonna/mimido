-- This file should undo anything in `up.sql`

ALTER TABLE `vtodos` DROP COLUMN `etag`;
ALTER TABLE `vevents` DROP COLUMN `etag`;
