-- This file should undo anything in `up.sql`
ALTER TABLE `vtodos` DROP COLUMN `starts_at`;
ALTER TABLE `vtodos` DROP COLUMN `ends_at`;
