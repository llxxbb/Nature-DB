use chrono::prelude::*;
use define::*;
use lazy_static::__Deref;
use nature_common::*;
use nature_common::util::id_tool::generate_id;
use nature_common::util::u128_to_vec_u8;
use serde::Serialize;
use std::fmt::Debug;
use super::super::schema::task;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Insertable, Queryable, QueryableByName)]
#[table_name = "task"]
pub struct RawTask {
    pub task_id: Vec<u8>,
    pub thing: String,
    pub data_type: i16,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub execute_time: NaiveDateTime,
    pub retried_times: i16,
}

impl RawTask {
    pub fn new<T: Serialize + Debug>(task: &T, thing: &str, data_type: i16) -> Result<RawTask> {
        let json = serde_json::to_string(task)?;
        if json.len() > *TASK_CONTENT_MAX_LENGTH.deref() {
            return Err(NatureError::DaoLogicalError("data's length can' be over : ".to_owned() + &TASK_CONTENT_MAX_LENGTH.to_string()));
        }
        let time = Local::now().naive_local();
        Ok(RawTask {
            task_id: u128_to_vec_u8(generate_id(&task)?),
            thing: thing.to_string(),
            data_type,
            data: json,
            create_time: time,
            execute_time: time,
            retried_times: 0,
        })
    }
}


