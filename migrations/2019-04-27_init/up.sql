-- Your SQL goes here
create TABLE `meta` (
	`full_key`	VARCHAR ( 255 ) NOT NULL,
	`description`	VARCHAR ( 1023 ),
	`version`	INTEGER NOT NULL,
	`states`	VARCHAR ( 1023 ),
	`fields`	VARCHAR ( 1023 ),
	`config`    VARCHAR(2048) DEFAULT '{}' NOT NULL,
	`flag`      INTEGER DEFAULT 1 NOT NULL,
	`create_time`	DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(`full_key`,`version`)
);

create TABLE `relation` (
	`from_meta`	VARCHAR ( 255 ) NOT NULL,
	`to_meta`	VARCHAR ( 255 ) NOT NULL,
	`settings`  VARCHAR ( 1023 ) NOT NULL,
	`flag`      INTEGER DEFAULT 1 NOT NULL,
	PRIMARY KEY(`from_meta`,`to_meta`)
);

create TABLE `instances` (
	`instance_id`	BINARY(16) NOT NULL,
	`meta`	VARCHAR ( 255 ) NOT NULL,
	`para`	VARCHAR ( 255 ) NOT NULL,
	`content`	VARCHAR ( 1023 ) NOT NULL,
	`context`	TEXT,
	`states`	TEXT,
	`state_version`	INTEGER NOT NULL,
	`from_meta`	VARCHAR ( 255 ),
	`from_id`	BINARY(16),
	`from_state_version`	INTEGER,
	`execute_time`	DATETIME NOT NULL,
	`create_time`	DATETIME NOT NULL,
	PRIMARY KEY(`meta`,`para`,`instance_id`,`state_version`)
);

create TABLE `task` (
	`task_id`	BINARY(16) NOT NULL,
	`meta`	VARCHAR ( 255 ) NOT NULL,
	`data_type`	SMALLINT NOT NULL,
	`data`	TEXT NOT NULL,
	`last_state_version`    INTEGER NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`execute_time`	DATETIME NOT NULL,
	`retried_times`	SMALLINT NOT NULL,
	PRIMARY KEY(`task_id`)
);

create TABLE `task_error` (
	`task_id`	BINARY(16) NOT NULL,
	`meta`	VARCHAR ( 255 ) NOT NULL,
	`data_type`	SMALLINT NOT NULL,
	`data`	TEXT NOT NULL,
	`last_state_version`    INTEGER NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`msg`	VARCHAR ( 255 ) NOT NULL,
	PRIMARY KEY(`task_id`)
);

create TABLE `plan` (
	`upstream`	VARCHAR ( 511 ) NOT NULL,
	`downstream`	VARCHAR ( 255 ) NOT NULL,
	`content`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	PRIMARY KEY(`upstream`,`downstream`)
);