use chrono::prelude::*;
use nature_common::*;
use sqlite::models::task::RawTask;
use super::super::schema::task_error;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "task_error"]
pub struct RawTaskError {
    pub task_id: Vec<u8>,
    pub thing: String,
    pub data_type: i16,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub msg: String,
}

impl RawTaskError {
    pub fn from_raw(err: &NatureError, raw: &RawTask) -> Self {
        RawTaskError {
            task_id: raw.task_id.clone(),
            thing: raw.thing.clone(),
            data_type: raw.data_type,
            data: raw.data.clone(),
            create_time: raw.create_time.clone(),
            msg: format!("{:?}", err),
        }
    }
}