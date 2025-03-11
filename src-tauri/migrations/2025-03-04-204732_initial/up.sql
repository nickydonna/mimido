-- Your SQL goes here
CREATE TABLE `todo_lists`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`ctag` TEXT NOT NULL
);

CREATE TABLE `events`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`calendar_id` INTEGER NOT NULL,
	`uid` TEXT NOT NULL,
	`etag` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`ical_data` TEXT NOT NULL,
	`last_modified` BIGINT NOT NULL,
	FOREIGN KEY (`calendar_id`) REFERENCES `calendars`(`id`)
);

CREATE TABLE `calendars`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`etag` TEXT,
	`server_id` INTEGER NOT NULL,
	FOREIGN KEY (`server_id`) REFERENCES `servers`(`id`)
);

CREATE TABLE `todos`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`list_id` INTEGER NOT NULL,
	`uid` TEXT NOT NULL,
	`etag` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`ical_data` TEXT NOT NULL,
	`last_modified` BIGINT NOT NULL,
	`completed` BOOL NOT NULL,
	FOREIGN KEY (`list_id`) REFERENCES `todo_lists`(`id`)
);

CREATE TABLE `servers`(
	`server_url` TEXT NOT NULL,
	`user` TEXT NOT NULL,
	`password` TEXT NOT NULL,
	`id` INTEGER NOT NULL PRIMARY KEY,
	`last_sync` BIGINT
);

