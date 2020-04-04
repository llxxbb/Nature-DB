use std::fmt::Debug;

use chrono::prelude::*;
use lazy_static::__Deref;
use serde::Serialize;

use nature_common::*;

use crate::models::define::*;

use super::super::schema::task;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Insertable, Queryable, QueryableByName)]
#[table_name = "task"]
pub struct RawTask {
    pub task_id: Vec<u8>,
    pub task_key: String,
    pub task_type: i8,
    pub task_for: String,
    pub task_state: i8,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub execute_time: NaiveDateTime,
    pub retried_times: i16,
}

impl RawTask {
    pub fn new<T: Serialize + Debug>(task: &T, task_key: &str, task_type: i8, task_for: &str) -> Result<RawTask> {
        let json = serde_json::to_string(task)?;
        Self::from_str(&json, task_key, task_type, task_for)
    }

    pub fn from_str(json: &str, task_key: &str, task_type: i8, task_for: &str) -> Result<RawTask> {
        if json.len() > *TASK_CONTENT_MAX_LENGTH.deref() {
            return Err(NatureError::SystemError("data's length can' be over : ".to_owned() + &TASK_CONTENT_MAX_LENGTH.to_string()));
        }
        let time = Local::now().naive_local();
        Ok(RawTask {
            task_id: u128_to_vec_u8(generate_id(json)?),
            task_key: task_key.to_string(),
            task_type,
            task_for: task_for.to_string(),
            task_state: 0,
            data: json.to_string(),
            create_time: time,
            execute_time: time,
            retried_times: 0,
        })
    }


    /// for performance reason, one-to-one carry which we can reuse the beginning carry to finish all flows.
    /// That way we need not to communicate with DB for create new and delete old carrier.
    /// But for failure we must redo from beginning. but I think it has small chance.
    /// Another disadvantage is the failure information will be attached to the beginning.
    pub fn finish_old<FI, FD>(&mut self, old: &RawTask, _dao_insert: FI, _dao_delete: FD) -> Result<usize>
        where FI: Fn(&RawTask) -> Result<usize>,
              FD: Fn(&[u8]) -> Result<usize>
    {
        // TODO  当遇到错误时如果要结束的 delivery ID 和新的delivery 不一样 需要结束之前的 delivery 并创建新的 delivery
        self.task_id = old.task_id.clone(); // the id is used for final finished
        Ok(1)
    }


    pub fn save_batch<FI, FD>(news: &[RawTask], old_id: &[u8], dao_insert: FI, dao_finish: FD) -> Result<()>
        where FI: Fn(&RawTask) -> Result<usize>,
              FD: Fn(&[u8]) -> Result<usize>
    {
        for v in news {
            dao_insert(v)?;
        }
        dao_finish(old_id)?;
        Ok(())
    }

    pub fn task_string(&self) -> String {
        format!("raw_task: key|type|for {}{}{}", self.task_key, self.task_type, self.task_for)
    }
}


