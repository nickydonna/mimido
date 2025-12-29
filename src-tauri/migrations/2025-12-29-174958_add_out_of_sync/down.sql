-- This file should undo anything in `up.sql`
ALTER TABLE `vevents` DROP COLUMN `out_of_sync`;
ALTER TABLE `vtodos` DROP COLUMN `out_of_sync`;
