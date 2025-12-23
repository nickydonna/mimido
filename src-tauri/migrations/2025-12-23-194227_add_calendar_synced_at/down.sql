-- This file should undo anything in `up.sql`
ALTER TABLE `calendars` DROP COLUMN `synced_at`;
