DROP TABLE IF EXISTS `todos`;
CREATE TABLE `todos`(
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

CREATE UNIQUE INDEX uid_todos ON todos(uid)


