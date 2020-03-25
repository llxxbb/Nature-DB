use chrono::prelude::*;

use nature_common::NatureError;

use crate::raw_models::RawTask;
use crate::schema::task_error;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "task_error"]
pub struct RawTaskError {
    pub task_id: Vec<u8>,
    pub task_key: String,
    pub task_type: i8,
    pub task_for: String,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub msg: String,
}

impl RawTaskError {
    pub fn from_raw(err: &NatureError, raw: &RawTask) -> Self {
        RawTaskError {
            task_id: raw.task_id.clone(),
            task_key: raw.task_key.clone(),
            task_type: raw.task_type,
            data: raw.data.clone(),
            create_time: raw.create_time,
            msg: format!("{:?}", err),
            task_for: "".to_string(),
        }
    }
}