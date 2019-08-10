use chrono::prelude::*;
use crate::schema::task_error;
use nature_common::NatureError;
use crate::raw_models::RawTask;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "task_error"]
pub struct RawTaskError {
    pub task_id: Vec<u8>,
    pub meta: String,
    pub data_type: i16,
    pub data: String,
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
            create_time: raw.create_time,
            msg: format!("{:?}", err),
        }
    }
}