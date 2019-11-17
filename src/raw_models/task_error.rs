use chrono::prelude::*;

use nature_common::NatureError;

use crate::raw_models::RawTask;
use crate::schema::task_error;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "task_error"]
pub struct RawTaskError {
    pub task_id: Vec<u8>,
    pub meta: String,
    pub data_type: i16,
    pub data: String,
    pub last_state_version: i32,
    pub create_time: NaiveDateTime,
    pub msg: String,
}

impl RawTaskError {
    pub fn from_raw(err: &NatureError, raw: &RawTask) -> Self {
        RawTaskError {
            task_id: raw.task_id.clone(),
            meta: raw.meta.clone(),
            data_type: raw.data_type,
            data: raw.data.clone(),
            last_state_version: raw.last_state_version,
            create_time: raw.create_time,
            msg: format!("{:?}", err),
        }
    }
}