-- This file should undo anything in `up.sql`
ALTER TABLE `vtodos` DROP COLUMN `has_rrule`;
ALTER TABLE `vtodos` DROP COLUMN `rrule_str`;

