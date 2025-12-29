-- Your SQL goes here
ALTER TABLE `vtodos` ADD COLUMN `out_of_sync` BOOL NOT NULL DEFAULT false;
ALTER TABLE `vevents` ADD COLUMN `out_of_sync` BOOL NOT NULL DEFAULT false;
