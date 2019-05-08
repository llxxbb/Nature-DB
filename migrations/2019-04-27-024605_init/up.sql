-- Your SQL goes here
create TABLE `thing_defines` (
	`full_key`	VARCHAR ( 255 ) NOT NULL,
	`description`	VARCHAR ( 1023 ),
	`version`	INTEGER NOT NULL,
	`states`	VARCHAR ( 1023 ),
	`fields`	VARCHAR ( 1023 ),
	`create_time`	DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(`full_key`,`version`)
);

create TABLE `one_step_flow` (
	`from_thing`	VARCHAR ( 255 ) NOT NULL,
	`from_version`	INTEGER NOT NULL,
	`to_thing`	VARCHAR ( 255 ) NOT NULL,
	`to_version`	INTEGER NOT NULL,
	`settings` VARCHAR ( 1023 ) NOT NULL,
	PRIMARY KEY(`from_thing`,`from_version`,`to_thing`,`to_version`)
);

create TABLE `instances` (
	`instance_id`	BINARY(16) NOT NULL,
	`thing`	VARCHAR ( 255 ) NOT NULL,
	`version`	INTEGER NOT NULL,
	`content`	VARCHAR ( 1023 ) NOT NULL,
	`context`	TEXT,
	`status`	TEXT,
	`status_version`	INTEGER NOT NULL,
	`from_thing`	VARCHAR ( 255 ),
	`from_version`	INTEGER,
	`from_status_version`	INTEGER,
	`event_time`	DATETIME NOT NULL,
	`execute_time`	DATETIME NOT NULL,
	`create_time`	DATETIME NOT NULL,
	PRIMARY KEY(`instance_id`,`thing`,`version`,`status_version`)
);

create TABLE `task` (
	`task_id`	BINARY(16) NOT NULL,
	`thing`	VARCHAR ( 255 ) NOT NULL,
	`data_type`	SMALLINT NOT NULL,
	`data`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`execute_time`	DATETIME NOT NULL,
	`retried_times`	SMALLINT NOT NULL,
	PRIMARY KEY(`task_id`)
);

create TABLE `task_error` (
	`task_id`	BINARY(16) NOT NULL,
	`thing`	VARCHAR ( 255 ) NOT NULL,
	`data_type`	SMALLINT NOT NULL,
	`data`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`msg`	VARCHAR ( 255 ) NOT NULL,
	PRIMARY KEY(`task_id`)
);

create TABLE `plan` (
	`upstream`	VARCHAR ( 511 ) NOT NULL,
	`to_biz`	VARCHAR ( 255 ) NOT NULL,
	`to_version`	INTEGER NOT NULL,
	`content`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	PRIMARY KEY(`upstream`,`to_biz`,`to_version`)
);