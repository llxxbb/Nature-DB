-- Your SQL goes here
create TABLE `meta` (
	`meta_type`	VARCHAR ( 10 ) NOT NULL,
	`meta_key`	VARCHAR ( 255 ) NOT NULL,
	`description`	VARCHAR ( 1023 ),
	`version`	INTEGER NOT NULL,
	`states`	VARCHAR ( 1023 ),
	`fields`	VARCHAR ( 1023 ),
	`config`    VARCHAR(2047) DEFAULT '{}' NOT NULL,
	`flag`      INTEGER DEFAULT 1 NOT NULL,
	`create_time`	DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(`meta_type`,`meta_key`,`version`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create TABLE `relation` (
	`from_meta`	VARCHAR ( 255 ) NOT NULL,
	`to_meta`	VARCHAR ( 255 ) NOT NULL,
	`settings`  VARCHAR ( 1023 ) NOT NULL,
	`flag`      INTEGER DEFAULT 1 NOT NULL,
	PRIMARY KEY(`from_meta`,`to_meta`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE `instances` (
  `ins_key` varchar(256) NOT NULL COMMENT 'meta|id|para',
  `content` text NOT NULL,
  `context` text DEFAULT NULL,
  `states` text DEFAULT NULL,
  `state_version` int(11) NOT NULL,
  `create_time` datetime NOT NULL,
  `from_key` varchar(256) NOT NULL COMMENT 'meta|id|para|sta_ver',
  PRIMARY KEY (`ins_key`,`state_version`),
  UNIQUE KEY `instances_un` (`ins_key`,`from_key`),
  KEY `instances_create_time_IDX` (`create_time`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create TABLE `task` (
	`task_id`	char(40) NOT NULL,
	`task_key`	VARCHAR ( 511 ) NOT NULL COMMENT 'meta|id|para|sta_ver',
	`task_type`	TINYINT NOT NULL,
	`task_for`	VARCHAR ( 255 ) NOT NULL,
	`task_state`	TINYINT NOT NULL,
	`data`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`execute_time`	DATETIME NOT NULL,
	`retried_times`	SMALLINT NOT NULL,
	UNIQUE KEY `task_un` (`task_key`,`task_type`,`task_for`),
	PRIMARY KEY(`task_id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create TABLE `task_error` (
	`task_id`	char(40) NOT NULL,
	`task_key`	VARCHAR ( 511 ) NOT NULL,
	`task_type`	TINYINT NOT NULL,
	`task_for`	VARCHAR ( 255 ) NOT NULL,
	`data`	TEXT NOT NULL,
	`create_time`	DATETIME NOT NULL,
	`msg`	VARCHAR ( 255 ) NOT NULL,
	UNIQUE KEY `task_un` (`task_key`,`task_type`,`task_for`),
	PRIMARY KEY(`task_id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
