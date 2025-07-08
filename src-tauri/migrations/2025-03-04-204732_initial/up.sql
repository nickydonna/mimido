CREATE TABLE `servers`(
	`server_url` TEXT NOT NULL,
	`user` TEXT NOT NULL,
	`password` TEXT NOT NULL,
	`id` INTEGER NOT NULL PRIMARY KEY,
	`last_sync` BIGINT
);

CREATE TABLE `calendars`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`etag` TEXT,
	`server_id` INTEGER NOT NULL,
	FOREIGN KEY (`server_id`) REFERENCES `servers`(`id`)
);

CREATE TABLE `vevents`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`calendar_id` INTEGER NOT NULL,
	`uid` TEXT NOT NULL,
	`ical_data` TEXT NOT NULL,
	`last_modified` BIGINT NOT NULL, 
	`summary` TEXT NOT NULL, 
	`href` TEXT NOT NULL, 
	`ends_at` TEXT NOT NULL, 
	`description` TEXT, 
	`starts_at` TEXT NOT NULL, 
	`event_type` TEXT NOT NULL, 
	`tag` TEXT, 
	`status` TEXT NOT NULL, 
	`original_text` TEXT, 
	`importance` INTEGER NOT NULL, 
	`load` INTEGER NOT NULL, 
	`urgency` INTEGER NOT NULL, 
	`postponed` INTEGER NOT NULL, 
	`has_rrule` INTEGER NOT NULL DEFAULT 0,
	`rrule_str` TEXT,
	FOREIGN KEY (`calendar_id`) REFERENCES `calendars`(`id`)
);
CREATE UNIQUE INDEX uid_event ON vevents(uid);
CREATE TABLE `vtodos`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`calendar_id` INTEGER NOT NULL,
	`uid` TEXT NOT NULL,
	`href` TEXT NOT NULL,
	`ical_data` TEXT NOT NULL,
	`last_modified` BIGINT NOT NULL,
	`completed` BOOL NOT NULL,
  `summary` TEXT NOT NULL,
  `description` TEXT,
  `event_type` TEXT NOT NULL DEFAULT "task",
  `tag` TEXT,
  `status` TEXT NOT NULL DEFAULT "todo",
  `original_text` TEXT,
  `importance` INTEGER NOT NULL DEFAULT 0,
  `load` INTEGER NOT NULL DEFAULT 0,
  `urgency` INTEGER NOT NULL DEFAULT 0,
  `postponed` INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY (`calendar_id`) REFERENCES `calendars`(`id`)
);
CREATE UNIQUE INDEX uid_todos ON vtodos(uid);
